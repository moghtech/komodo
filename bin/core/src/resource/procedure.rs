use std::time::Duration;

use anyhow::{Context, anyhow};
use database::mungos::{
  find::find_collect,
  mongodb::{Collection, bson::doc, options::FindOneOptions},
};
use futures_util::{TryStreamExt, stream::FuturesUnordered};
use komodo_client::{
  api::execute::Execution,
  entities::{
    Operation, ResourceTarget, ResourceTargetVariant,
    action::Action,
    alerter::Alerter,
    build::Build,
    deployment::Deployment,
    permission::PermissionLevel,
    procedure::{
      PartialProcedureConfig, Procedure, ProcedureConfig,
      ProcedureConfigDiff, ProcedureListItem, ProcedureListItemInfo,
      ProcedureQuerySpecifics, ProcedureState,
    },
    repo::Repo,
    resource::Resource,
    server::Server,
    stack::Stack,
    swarm::Swarm,
    sync::ResourceSync,
    update::Update,
    user::User,
  },
};

use crate::{
  config::core_config,
  helpers::query::{get_last_run_at, get_procedure_state},
  schedule::{
    cancel_schedule, get_schedule_item_info, update_schedule,
  },
  state::{action_states, db_client, procedure_state_cache},
};

impl super::KomodoResource for Procedure {
  type Config = ProcedureConfig;
  type PartialConfig = PartialProcedureConfig;
  type ConfigDiff = ProcedureConfigDiff;
  type Info = ();
  type ListItem = ProcedureListItem;
  type QuerySpecifics = ProcedureQuerySpecifics;

  fn resource_type() -> ResourceTargetVariant {
    ResourceTargetVariant::Procedure
  }

  fn resource_target(id: impl Into<String>) -> ResourceTarget {
    ResourceTarget::Procedure(id.into())
  }

  fn coll() -> &'static Collection<Resource<Self::Config, Self::Info>>
  {
    &db_client().procedures
  }

  async fn to_list_item(
    procedure: Resource<Self::Config, Self::Info>,
  ) -> Self::ListItem {
    let (state, last_run_at) = tokio::join!(
      get_procedure_state(&procedure.id),
      get_last_run_at::<Procedure>(&procedure.id)
    );
    let (next_scheduled_run, schedule_error) = get_schedule_item_info(
      &ResourceTarget::Procedure(procedure.id.clone()),
    );
    ProcedureListItem {
      name: procedure.name,
      id: procedure.id,
      template: procedure.template,
      tags: procedure.tags,
      resource_type: ResourceTargetVariant::Procedure,
      info: ProcedureListItemInfo {
        stages: procedure.config.stages.len() as i64,
        state,
        last_run_at: last_run_at.unwrap_or(None),
        next_scheduled_run,
        schedule_error,
      },
    }
  }

  async fn busy(id: &String) -> anyhow::Result<bool> {
    action_states()
      .procedure
      .get(id)
      .await
      .unwrap_or_default()
      .busy()
  }

  // CREATE

  fn create_operation() -> Operation {
    Operation::CreateProcedure
  }

  fn user_can_create(user: &User) -> bool {
    user.admin || !core_config().disable_non_admin_create
  }

  async fn validate_create_config(
    config: &mut Self::PartialConfig,
    user: &User,
  ) -> anyhow::Result<()> {
    validate_config(config, user, None).await
  }

  async fn post_create(
    created: &Resource<Self::Config, Self::Info>,
    _update: &mut Update,
  ) -> anyhow::Result<()> {
    update_schedule(created);
    refresh_procedure_state_cache().await;
    Ok(())
  }

  // UPDATE

  fn update_operation() -> Operation {
    Operation::UpdateProcedure
  }

  async fn validate_update_config(
    id: &str,
    config: &mut Self::PartialConfig,
    user: &User,
  ) -> anyhow::Result<()> {
    validate_config(config, user, Some(id)).await
  }

  async fn post_update(
    updated: &Self,
    update: &mut Update,
  ) -> anyhow::Result<()> {
    Self::post_create(updated, update).await
  }

  // RENAME

  fn rename_operation() -> Operation {
    Operation::RenameProcedure
  }

  // DELETE

  fn delete_operation() -> Operation {
    Operation::DeleteProcedure
  }

  async fn pre_delete(
    _resource: &Resource<Self::Config, Self::Info>,
    _update: &mut Update,
  ) -> anyhow::Result<()> {
    Ok(())
  }

  async fn post_delete(
    resource: &Resource<Self::Config, Self::Info>,
    _update: &mut Update,
  ) -> anyhow::Result<()> {
    cancel_schedule(&ResourceTarget::Procedure(resource.id.clone()));
    procedure_state_cache().remove(&resource.id).await;
    Ok(())
  }
}

#[instrument("ValidateProcedureConfig", skip_all)]
async fn validate_config(
  config: &mut PartialProcedureConfig,
  user: &User,
  id: Option<&str>,
) -> anyhow::Result<()> {
  let Some(stages) = &mut config.stages else {
    return Ok(());
  };
  for stage in stages {
    for exec in &mut stage.executions {
      macro_rules! check_execution_perms {
        (
          execute: [$(($Variant:ident, $Type:ident, $field:ident)),* $(,)?],
          batch_admin: [$($BatchVariant:ident),* $(,)?],
          admin_only: [$(($AdminVariant:ident, $msg:literal)),* $(,)?],
        ) => {
          match &mut exec.execution {
            $(
              Execution::$Variant(params) => {
                let resource = super::get_check_permissions::<$Type>(
                  &params.$field,
                  user,
                  PermissionLevel::Execute.into(),
                )
                .await?;
                params.$field = resource.id;
              }
            )*
            $(
              Execution::$BatchVariant(_params) => {
                if !user.admin {
                  return Err(anyhow!(
                    "Non admin user cannot configure Batch executions"
                  ));
                }
              }
            )*
            $(
              Execution::$AdminVariant(_params) => {
                if !user.admin {
                  return Err(anyhow!($msg));
                }
              }
            )*
            // Special: self-referential procedure check
            Execution::RunProcedure(params) => {
              let procedure = super::get_check_permissions::<Procedure>(
                &params.procedure,
                user,
                PermissionLevel::Execute.into(),
              )
              .await?;
              match id {
                Some(id) if procedure.id == id => {
                  return Err(anyhow!(
                    "Cannot have self-referential procedure"
                  ));
                }
                _ => {}
              }
              params.procedure = procedure.id;
            }
            // Special: CommitSync uses Write permission
            Execution::CommitSync(params) => {
              let sync = super::get_check_permissions::<ResourceSync>(
                &params.sync,
                user,
                PermissionLevel::Write.into(),
              )
              .await?;
              params.sync = sync.id;
            }
            // Special: SendAlert checks a Vec of alerters
            Execution::SendAlert(params) => {
              params.alerters = params
                .alerters
                .iter()
                .map(async |alerter| {
                  let id = super::get_check_permissions::<Alerter>(
                    alerter,
                    user,
                    PermissionLevel::Execute.into(),
                  )
                  .await?
                  .id;
                  anyhow::Ok(id)
                })
                .collect::<FuturesUnordered<_>>()
                .try_collect::<Vec<_>>()
                .await?;
            }
            Execution::None(_) | Execution::Sleep(_) => {}
          }
        };
      }
      check_execution_perms!(
        execute: [
          // Action
          (RunAction, Action, action),
          // Build
          (RunBuild, Build, build),
          (CancelBuild, Build, build),
          // Deployment
          (Deploy, Deployment, deployment),
          (PullDeployment, Deployment, deployment),
          (StartDeployment, Deployment, deployment),
          (RestartDeployment, Deployment, deployment),
          (PauseDeployment, Deployment, deployment),
          (UnpauseDeployment, Deployment, deployment),
          (StopDeployment, Deployment, deployment),
          (DestroyDeployment, Deployment, deployment),
          // Repo
          (CloneRepo, Repo, repo),
          (PullRepo, Repo, repo),
          (BuildRepo, Repo, repo),
          (CancelRepoBuild, Repo, repo),
          // Server
          (StartContainer, Server, server),
          (RestartContainer, Server, server),
          (PauseContainer, Server, server),
          (UnpauseContainer, Server, server),
          (StopContainer, Server, server),
          (DestroyContainer, Server, server),
          (StartAllContainers, Server, server),
          (RestartAllContainers, Server, server),
          (PauseAllContainers, Server, server),
          (UnpauseAllContainers, Server, server),
          (StopAllContainers, Server, server),
          (PruneContainers, Server, server),
          (DeleteNetwork, Server, server),
          (PruneNetworks, Server, server),
          (DeleteImage, Server, server),
          (PruneImages, Server, server),
          (DeleteVolume, Server, server),
          (PruneVolumes, Server, server),
          (PruneDockerBuilders, Server, server),
          (PruneBuildx, Server, server),
          (PruneSystem, Server, server),
          // Resource Sync
          (RunSync, ResourceSync, sync),
          // Stack
          (DeployStack, Stack, stack),
          (DeployStackIfChanged, Stack, stack),
          (PullStack, Stack, stack),
          (StartStack, Stack, stack),
          (RestartStack, Stack, stack),
          (PauseStack, Stack, stack),
          (UnpauseStack, Stack, stack),
          (StopStack, Stack, stack),
          (DestroyStack, Stack, stack),
          (RunStackService, Stack, stack),
          // Alerter
          (TestAlerter, Alerter, alerter),
          // Swarm
          (RemoveSwarmNodes, Swarm, swarm),
          (UpdateSwarmNode, Swarm, swarm),
          (RemoveSwarmStacks, Swarm, swarm),
          (RemoveSwarmServices, Swarm, swarm),
          (CreateSwarmConfig, Swarm, swarm),
          (RotateSwarmConfig, Swarm, swarm),
          (RemoveSwarmConfigs, Swarm, swarm),
          (CreateSwarmSecret, Swarm, swarm),
          (RotateSwarmSecret, Swarm, swarm),
          (RemoveSwarmSecrets, Swarm, swarm),
        ],
        batch_admin: [
          BatchRunProcedure,
          BatchRunAction,
          BatchRunBuild,
          BatchDeploy,
          BatchDestroyDeployment,
          BatchCloneRepo,
          BatchPullRepo,
          BatchBuildRepo,
          BatchDeployStack,
          BatchDeployStackIfChanged,
          BatchPullStack,
          BatchDestroyStack,
        ],
        admin_only: [
          (ClearRepoCache, "Non admin user cannot clear repo cache"),
          (BackupCoreDatabase, "Non admin user cannot trigger core database backup"),
          (GlobalAutoUpdate, "Non admin user cannot trigger global auto update"),
          (RotateAllServerKeys, "Non admin user cannot trigger rotate all server keys"),
          (RotateCoreKeys, "Non admin user cannot trigger rotate core keys"),
        ],
      );
    }
  }

  Ok(())
}

pub fn spawn_procedure_state_refresh_loop() {
  tokio::spawn(async move {
    loop {
      refresh_procedure_state_cache().await;
      tokio::time::sleep(Duration::from_secs(60)).await;
    }
  });
}

pub async fn refresh_procedure_state_cache() {
  let _ = async {
    let procedures =
      find_collect(&db_client().procedures, None, None)
        .await
        .context("Failed to get Procedures from db")?;
    let cache = procedure_state_cache();
    for procedure in procedures {
      let state = get_procedure_state_from_db(&procedure.id).await;
      cache.insert(procedure.id, state).await;
    }
    anyhow::Ok(())
  }
  .await
  .inspect_err(|e| {
    error!("Failed to refresh Procedure state cache | {e:#}")
  });
}

async fn get_procedure_state_from_db(id: &str) -> ProcedureState {
  async {
    let state = db_client()
      .updates
      .find_one(doc! {
        "target.type": "Procedure",
        "target.id": id,
        "operation": "RunProcedure"
      })
      .with_options(
        FindOneOptions::builder()
          .sort(doc! { "start_ts": -1 })
          .build(),
      )
      .await?
      .map(|u| {
        if u.success {
          ProcedureState::Ok
        } else {
          ProcedureState::Failed
        }
      })
      .unwrap_or(ProcedureState::Ok);
    anyhow::Ok(state)
  }
  .await
  .inspect_err(|e| {
    warn!("Failed to get Procedure state for {id} | {e:#}")
  })
  .unwrap_or(ProcedureState::Unknown)
}
