use std::str::FromStr;

use colored::Colorize;
use database::mungos::{
  find::find_collect,
  mongodb::bson::{Document, doc, oid::ObjectId, to_document},
};
use futures::future::join_all;
use komodo_client::{
  api::{
    auth::SignUpLocalUser,
    execute::{
      BackupCoreDatabase, Execution, GlobalAutoUpdate, RunAction,
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
    server::{PartialServerConfig, Server},
    sync::ResourceSync,
    tag::TagColor,
    update::Log,
    user::{action_user, system_user},
  },
};
use resolver_api::Resolve;

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
    clean_up_server_templates(),
    ensure_first_server_and_builder(),
    ensure_init_user_and_resources(),
  );
}

async fn in_progress_update_cleanup() {
  let log = Log::error(
    "Komodo shutdown",
    String::from(
      "Komodo shutdown during execution. If this is a build, the builder may not have been terminated.",
    ),
  );
  // This static log won't fail to serialize, unwrap ok.
  let log = to_document(&log).unwrap();
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
  let Some(address) = config.first_server.clone() else {
    return;
  };
  let db = db_client();
  let Ok(server) = db
    .servers
    .find_one(Document::new())
    .await
    .inspect_err(|e| error!("Failed to initialize 'first_server'. Failed to query db. {e:?}"))
  else {
    return;
  };
  let server = if let Some(server) = server {
    server
  } else {
    match (CreateServer {
      name: config.first_server_name.clone(),
      config: PartialServerConfig {
        address: Some(address),
        enabled: Some(true),
        ..Default::default()
      },
    })
    .resolve(&WriteArgs {
      user: system_user().to_owned(),
    })
    .await
    {
      Ok(server) => server,
      Err(e) => {
        error!(
          "Failed to initialize 'first_server'. Failed to CreateServer. {:#}",
          e.error
        );
        return;
      }
    }
  };
  let Ok(None) = db.builders
    .find_one(Document::new()).await
    .inspect_err(|e| error!("Failed to initialize 'first_builder' | Failed to query db | {e:?}")) else {
      return;
    };
  if let Err(e) = (CreateBuilder {
    name: String::from("Local"),
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
    SignUpLocalUser {
      username: username.clone(),
      password: config.init_admin_password.clone(),
    }
    .resolve(&AuthArgs::default())
    .await
    .expect("Failed to initialize default admin user.");
    db.users
      .find_one(doc! { "username": username })
      .await
      .expect("Failed to query database for initial user")
      .expect("Failed to find initial user after creation");
  };

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
}

/// v1.17.5 removes the ServerTemplate resource.
/// References to this resource type need to be cleaned up
/// to avoid type errors reading from the database.
async fn clean_up_server_templates() {
  let db = db_client();
  tokio::join!(
    async {
      db.permissions
        .delete_many(doc! {
          "resource_target.type": "ServerTemplate",
        })
        .await
        .expect(
          "Failed to clean up server template permissions on db",
        );
    },
    async {
      db.updates
        .delete_many(doc! { "target.type": "ServerTemplate" })
        .await
        .expect("Failed to clean up server template updates on db");
    },
    async {
      db.users
        .update_many(
          Document::new(),
          doc! { "$unset": { "recents.ServerTemplate": 1, "all.ServerTemplate": 1 } }
        )
        .await
        .expect("Failed to clean up server template updates on db");
    },
    async {
      db.user_groups
        .update_many(
          Document::new(),
          doc! { "$unset": { "all.ServerTemplate": 1 } },
        )
        .await
        .expect("Failed to clean up server template updates on db");
    },
  );
}
