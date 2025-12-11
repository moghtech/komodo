use std::{
  net::{IpAddr, Ipv4Addr},
  str::FromStr,
};

use anyhow::Context;
use colored::Colorize;
use database::mungos::{
  find::find_collect,
  mongodb::bson::{
    Document, doc, oid::ObjectId, to_bson, to_document,
  },
};
use futures_util::future::join_all;
use komodo_client::{
  api::{
    auth::SignUpLocalUser,
    execute::{
      BackupCoreDatabase, Execution, GlobalAutoUpdate,
      RotateAllServerKeys, RunAction,
    },
    write::{
      CreateBuilder, CreateProcedure, CreateServer, CreateTag,
      UpdateResourceMeta,
    },
  },
  entities::{
    ResourceTarget,
    builder::{PartialBuilderConfig, PartialServerBuilderConfig},
    komodo_timestamp,
    procedure::{EnabledExecution, ProcedureConfig, ProcedureStage},
    server::{PartialServerConfig, Server, ServerInfo},
    sync::ResourceSync,
    tag::TagColor,
    update::Log,
    user::{action_user, system_user},
  },
};
use resolver_api::Resolve;
use uuid::Uuid;

use crate::{
  api::{
    auth::AuthArgs,
    execute::{ExecuteArgs, ExecuteRequest},
    write::WriteArgs,
  },
  config::core_config,
  helpers::update::init_execution_update,
  network, resource,
  state::db_client,
};

/// Runs the Actions with `run_at_startup: true`
pub async fn run_startup_actions() {
  let startup_actions = match find_collect(
    &db_client().actions,
    doc! { "config.run_at_startup": true },
    None,
  )
  .await
  {
    Ok(actions) => actions,
    Err(e) => {
      error!("Failed to fetch actions for startup | {e:#?}");
      return;
    }
  };

  for action in startup_actions {
    let name = action.name;
    let id = action.id;
    let update = match init_execution_update(
      &ExecuteRequest::RunAction(RunAction {
        action: name.clone(),
        args: Default::default(),
      }),
      action_user(),
    )
    .await
    {
      Ok(update) => update,
      Err(e) => {
        error!(
          "Failed to initialize update for action {name} ({id}) | {e:#?}"
        );
        continue;
      }
    };

    if let Err(e) = (RunAction {
      action: name.clone(),
      args: Default::default(),
    })
    .resolve(&ExecuteArgs {
      user: action_user().to_owned(),
      update,
      id: Uuid::new_v4(),
    })
    .await
    {
      error!(
        "Failed to execute startup action {name} ({id}) | {e:#?}"
      );
    }
  }
}

/// This function should be run on startup,
/// after the db client has been initialized
pub async fn on_startup() {
  // Configure manual network interface if specified
  network::configure_internet_gateway().await;

  tokio::join!(
    in_progress_update_cleanup(),
    open_alert_cleanup(),
    ensure_first_server_and_builder(),
    ensure_init_user_and_resources(),
    clean_up_server_templates(),
    init_server_info(),
  );
}

async fn in_progress_update_cleanup() {
  let log = Log::error(
    "Komodo shutdown",
    String::from(
      "Komodo shutdown during execution. If this is a build, the builder may not have been terminated.",
    ),
  );
  let log = match to_document(&log)
    .context("Failed to serialize log to document")
  {
    Ok(log) => log,
    Err(e) => {
      error!("Failed to cleanup in progress update | {e:#}");
      return;
    }
  };
  if let Err(e) = db_client()
    .updates
    .update_many(
      doc! { "status": "InProgress" },
      doc! {
        "$set": {
          "status": "Complete",
          "success": false,
        },
        "$push": {
          "logs": log
        }
      },
    )
    .await
  {
    error!("failed to cleanup in progress updates on startup | {e:#}")
  }
}

/// Run on startup, ensure open alerts pointing to invalid resources are closed.
async fn open_alert_cleanup() {
  let db = db_client();
  let Ok(alerts) =
    find_collect(&db.alerts, doc! { "resolved": false }, None)
      .await
      .inspect_err(|e| {
        error!(
          "failed to list all alerts for startup open alert cleanup | {e:?}"
        )
      })
  else {
    return;
  };
  let futures = alerts.into_iter().map(|alert| async move {
    match alert.target {
      ResourceTarget::Server(id) => {
        resource::get::<Server>(&id)
          .await
          .is_err()
          .then(|| ObjectId::from_str(&alert.id).inspect_err(|e| warn!("failed to clean up alert - id is invalid ObjectId | {e:?}")).ok()).flatten()
      }
      ResourceTarget::ResourceSync(id) => {
        resource::get::<ResourceSync>(&id)
          .await
          .is_err()
          .then(|| ObjectId::from_str(&alert.id).inspect_err(|e| warn!("failed to clean up alert - id is invalid ObjectId | {e:?}")).ok()).flatten()
      }
      // No other resources should have open alerts.
      _ => ObjectId::from_str(&alert.id).inspect_err(|e| warn!("failed to clean up alert - id is invalid ObjectId | {e:?}")).ok(),
    }
  });
  let to_update_ids = join_all(futures)
    .await
    .into_iter()
    .flatten()
    .collect::<Vec<_>>();
  if let Err(e) = db
    .alerts
    .update_many(
      doc! { "_id": { "$in": to_update_ids } },
      doc! { "$set": {
        "resolved": true,
        "resolved_ts": komodo_timestamp()
      } },
    )
    .await
  {
    error!(
      "failed to clean up invalid open alerts on startup | {e:#}"
    )
  }
}

/// Ensures a default server / builder exists with the defined address
async fn ensure_first_server_and_builder() {
  let config = core_config();
  if config.first_server_name.is_none()
    && config.first_server_address.is_none()
  {
    // If neither defined, early return
    return;
  }
  // Maybe create first Server / Builder
  let db = db_client();
  // If any Server exists, exit early.
  let Ok(None) =
    db.servers.find_one(Document::new()).await.inspect_err(|e| {
      error!(
        "Failed to initialize first Server. Failed to query db. {e:?}"
      )
    })
  else {
    return;
  };
  // Use the same name for Server and Builder
  let name = config.first_server_name.as_deref().unwrap_or("Local");
  let server = match (CreateServer {
    name: name.to_string(),
    config: PartialServerConfig {
      address: config.first_server_address.clone(),
      enabled: Some(true),
      ..Default::default()
    },
    public_key: None,
  })
  .resolve(&WriteArgs {
    user: system_user().to_owned(),
  })
  .await
  {
    Ok(server) => server,
    Err(e) => {
      error!(
        "Failed to initialize first Server. Failed to CreateServer. {:#}",
        e.error
      );
      return;
    }
  };
  // If any builder exists, exit early.
  let Ok(None) = db.builders
    .find_one(Document::new()).await
    .inspect_err(|e| error!("Failed to initialize 'first_builder' | Failed to query db | {e:?}")) else {
      return;
    };
  if let Err(e) = (CreateBuilder {
    // Same name as Server
    name: name.to_string(),
    config: PartialBuilderConfig::Server(
      PartialServerBuilderConfig {
        server_id: Some(server.id),
      },
    ),
  })
  .resolve(&WriteArgs {
    user: system_user().to_owned(),
  })
  .await
  {
    error!(
      "Failed to initialize 'first_builder' | Failed to CreateBuilder | {:#}",
      e.error
    );
  }
}

async fn ensure_init_user_and_resources() {
  let db = db_client();

  // Assumes if there are any existing users, procedures, or tags,
  // the default procedures do not need to be set up.
  let Ok((None, None, None)) = tokio::try_join!(
    db.users.find_one(Document::new()),
    db.procedures.find_one(Document::new()),
    db.tags.find_one(Document::new()),
  ).inspect_err(|e| error!("Failed to initialize default procedures | Failed to query db | {e:?}")) else {
    return
  };

  let config = core_config();

  // Init admin user if set in config.
  if let Some(username) = &config.init_admin_username {
    info!("Creating init admin user...");
    if let Err(e) = (SignUpLocalUser {
      username: username.clone(),
      password: config.init_admin_password.clone(),
    })
    .resolve(&AuthArgs {
      headers: Default::default(),
      ip: IpAddr::V4(Ipv4Addr::UNSPECIFIED),
      session: None,
    })
    .await
    {
      error!("Failed to create init admin user | {:#}", e.error);
      return;
    }
    match db
      .users
      .find_one(doc! { "username": username })
      .await
      .context(
        "Failed to query database for init admin user after creation",
      ) {
      Ok(Some(_)) => {
        info!("Successfully created init admin user.")
      }
      Ok(None) => {
        error!("Failed to find init admin user after creation");
      }
      Err(e) => {
        error!("{e:#}");
      }
    }
  }

  if config.disable_init_resources {
    info!("System resources init {}", "DISABLED".red());
    return;
  }

  info!("Creating init system resources...");

  let write_args = WriteArgs {
    user: system_user().to_owned(),
  };

  // Create default 'system' tag
  let default_tags = match (CreateTag {
    name: String::from("system"),
    color: Some(TagColor::Red),
  })
  .resolve(&write_args)
  .await
  {
    Ok(tag) => vec![tag.id],
    Err(e) => {
      warn!("Failed to create default tag | {:#}", e.error);
      Vec::new()
    }
  };

  // Backup Core Database
  async {
    let Ok(config) = ProcedureConfig::builder()
      .stages(vec![ProcedureStage {
        name: String::from("Stage 1"),
        enabled: true,
        executions: vec![
          EnabledExecution {
            execution: Execution::BackupCoreDatabase(BackupCoreDatabase {}),
            enabled: true
          }
        ]
      }])
      .schedule(String::from("Every day at 01:00"))
      .build()
      .inspect_err(|e| error!("Failed to initialize backup core database procedure | Failed to build Procedure | {e:?}")) else {
      return;
    };
    let procedure = match (CreateProcedure {
      name: String::from("Backup Core Database"),
      config: config.into()
    }).resolve(&write_args).await {
      Ok(procedure) => procedure,
      Err(e) => {
        error!(
          "Failed to initialize default database backup Procedure | Failed to create Procedure | {:#}",
          e.error
        );
        return;
      }
    };
    if let Err(e) = (UpdateResourceMeta {
      target: ResourceTarget::Procedure(procedure.id),
      tags: Some(default_tags.clone()),
      description: Some(String::from(
        "Triggers the Core database backup at the scheduled time.",
      )),
      template: None,
    }).resolve(&write_args).await {
      warn!("Failed to update default database backup Procedure tags / description | {:#}", e.error);
    }
  }.await;

  // GlobalAutoUpdate
  async {
    let Ok(config) = ProcedureConfig::builder()
      .stages(vec![ProcedureStage {
        name: String::from("Stage 1"),
        enabled: true,
        executions: vec![
          EnabledExecution {
            execution: Execution::GlobalAutoUpdate(GlobalAutoUpdate {}),
            enabled: true
          }
        ]
      }])
      .schedule(String::from("Every day at 03:00"))
      .build()
      .inspect_err(|e| error!("Failed to initialize global auto update procedure | Failed to build Procedure | {e:?}")) else {
      return;
    };
    let procedure = match (CreateProcedure {
      name: String::from("Global Auto Update"),
      config: config.into(),
    })
    .resolve(&write_args)
    .await
    {
      Ok(procedure) => procedure,
      Err(e) => {
        error!(
          "Failed to initialize global auto update Procedure | Failed to create Procedure | {:#}",
          e.error
        );
        return;
      }
    };
    if let Err(e) = (UpdateResourceMeta {
      target: ResourceTarget::Procedure(procedure.id),
      tags: Some(default_tags.clone()),
      description: Some(String::from(
        "Pulls and auto updates Stacks and Deployments using 'poll_for_updates' or 'auto_update'.",
      )),
      template: None,
    })
    .resolve(&write_args)
    .await
    {
      warn!(
        "Failed to update global auto update Procedure tags / description | {:#}",
        e.error
      );
    }
  }.await;

  // RotateAllServerKeys
  async {
    let Ok(config) = ProcedureConfig::builder()
      .stages(vec![ProcedureStage {
        name: String::from("Stage 1"),
        enabled: true,
        executions: vec![
          EnabledExecution {
            execution: Execution::RotateAllServerKeys(RotateAllServerKeys {}),
            enabled: true
          }
        ]
      }])
      .schedule(String::from("Every day at 06:00"))
      .build()
      .inspect_err(|e| error!("Failed to initialize Server key rotation Procedure | Failed to build Procedure | {e:?}")) else {
      return;
    };
    let procedure = match (CreateProcedure {
      name: String::from("Rotate Server Keys"),
      config: config.into(),
    })
    .resolve(&write_args)
    .await
    {
      Ok(procedure) => procedure,
      Err(e) => {
        error!(
          "Failed to initialize Server key rotation Procedure | Failed to create Procedure | {:#}",
          e.error
        );
        return;
      }
    };
    if let Err(e) = (UpdateResourceMeta {
      target: ResourceTarget::Procedure(procedure.id),
      tags: Some(default_tags.clone()),
      description: Some(String::from(
        "Rotates all currently connected Server keys.",
      )),
      template: None,
    })
    .resolve(&write_args)
    .await
    {
      warn!(
        "Failed to update Server key rotation Procedure tags / description | {:#}",
        e.error
      );
    }
  }.await;
}

/// v1.17.5 removes the ServerTemplate resource.
/// References to this resource type need to be cleaned up
/// to avoid type errors reading from the database.
async fn clean_up_server_templates() {
  let db = db_client();
  tokio::join!(
    async {
      if let Err(e) = db
        .permissions
        .delete_many(doc! {
          "resource_target.type": "ServerTemplate",
        })
        .await
      {
        error!(
          "Failed to clean up server template permissions on database | {e:#}"
        );
      }
    },
    async {
      if let Err(e) = db
        .updates
        .delete_many(doc! { "target.type": "ServerTemplate" })
        .await
      {
        error!(
          "Failed to clean up server template updates on database | {e:#}"
        );
      }
    },
    async {
      if let Err(e) = db.users
        .update_many(
          Document::new(),
          doc! { "$unset": { "recents.ServerTemplate": 1, "all.ServerTemplate": 1 } }
        )
        .await
      {
        error!(
          "Failed to clean up server template user references on database | {e:#}"
        );
      }
    },
    async {
      if let Err(e) = db
        .user_groups
        .update_many(
          Document::new(),
          doc! { "$unset": { "all.ServerTemplate": 1 } },
        )
        .await
      {
        error!(
          "Failed to clean up server template user group references on database | {e:#}"
        );
      }
    },
  );
}

/// v2 adds ServerInfo to ServerSchema.
/// Need to ensure it is initialized from null.
async fn init_server_info() {
  let default_server_info = match to_bson(&ServerInfo::default()) {
    Ok(info) => info,
    Err(e) => {
      error!("Failed to serialize ServerInfo to bson | {e:?}");
      return;
    }
  };
  if let Err(e) = db_client()
    .servers
    .update_many(
      doc! { "info": null },
      doc! { "$set": { "info": default_server_info } },
    )
    .await
  {
    error!("Failed to migrate ServerInfo to v2 | {e:?}");
  }
}
