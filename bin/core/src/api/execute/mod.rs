use std::pin::Pin;

use anyhow::Context;
use axum::{
  Extension, Router, extract::Path, middleware, routing::post,
};
use axum_extra::{TypedHeader, headers::ContentType};
use database::mungos::by_id::find_one_by_id;
use derive_variants::{EnumVariants, ExtractVariant};
use formatting::format_serror;
use futures_util::future::join_all;
use komodo_client::{
  api::execute::*,
  entities::{
    Operation,
    permission::PermissionLevel,
    update::{Log, Update},
    user::User,
  },
};
use resolver_api::Resolve;
use response::JsonString;
use serde::{Deserialize, Serialize};
use serde_json::json;
use serror::Json;
use strum::Display;
use typeshare::typeshare;
use uuid::Uuid;

use crate::{
  auth::auth_request,
  helpers::update::{init_execution_update, update_update},
  resource::{KomodoResource, list_full_for_user_using_pattern},
  state::db_client,
};

mod action;
mod alerter;
mod build;
mod deployment;
mod maintenance;
mod procedure;
mod repo;
mod server;
mod stack;
mod swarm;
mod sync;

use super::Variant;

pub use {
  deployment::pull_deployment_inner, stack::pull_stack_inner,
};

pub struct ExecuteArgs {
  /// The execution id.
  /// Unique for every /execute call.
  pub id: Uuid,
  pub user: User,
  pub update: Update,
}

#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EnumVariants,
)]
#[variant_derive(Debug, Display)]
#[args(ExecuteArgs)]
#[response(JsonString)]
#[error(serror::Error)]
#[serde(tag = "type", content = "params")]
pub enum ExecuteRequest {
  // ==== STACK ====
  DeployStack(DeployStack),
  BatchDeployStack(BatchDeployStack),
  DeployStackIfChanged(DeployStackIfChanged),
  BatchDeployStackIfChanged(BatchDeployStackIfChanged),
  PullStack(PullStack),
  BatchPullStack(BatchPullStack),
  StartStack(StartStack),
  RestartStack(RestartStack),
  StopStack(StopStack),
  PauseStack(PauseStack),
  UnpauseStack(UnpauseStack),
  DestroyStack(DestroyStack),
  BatchDestroyStack(BatchDestroyStack),
  RunStackService(RunStackService),

  // ==== DEPLOYMENT ====
  Deploy(Deploy),
  BatchDeploy(BatchDeploy),
  PullDeployment(PullDeployment),
  StartDeployment(StartDeployment),
  RestartDeployment(RestartDeployment),
  PauseDeployment(PauseDeployment),
  UnpauseDeployment(UnpauseDeployment),
  StopDeployment(StopDeployment),
  DestroyDeployment(DestroyDeployment),
  BatchDestroyDeployment(BatchDestroyDeployment),

  // ==== BUILD ====
  RunBuild(RunBuild),
  BatchRunBuild(BatchRunBuild),
  CancelBuild(CancelBuild),

  // ==== REPO ====
  CloneRepo(CloneRepo),
  BatchCloneRepo(BatchCloneRepo),
  PullRepo(PullRepo),
  BatchPullRepo(BatchPullRepo),
  BuildRepo(BuildRepo),
  BatchBuildRepo(BatchBuildRepo),
  CancelRepoBuild(CancelRepoBuild),

  // ==== PROCEDURE ====
  RunProcedure(RunProcedure),
  BatchRunProcedure(BatchRunProcedure),

  // ==== ACTION ====
  RunAction(RunAction),
  BatchRunAction(BatchRunAction),

  // ==== SYNC ====
  RunSync(RunSync),

  // ==== ALERTER ====
  TestAlerter(TestAlerter),
  SendAlert(SendAlert),

  // ==== SERVER ====
  StartContainer(StartContainer),
  RestartContainer(RestartContainer),
  PauseContainer(PauseContainer),
  UnpauseContainer(UnpauseContainer),
  StopContainer(StopContainer),
  DestroyContainer(DestroyContainer),
  StartAllContainers(StartAllContainers),
  RestartAllContainers(RestartAllContainers),
  PauseAllContainers(PauseAllContainers),
  UnpauseAllContainers(UnpauseAllContainers),
  StopAllContainers(StopAllContainers),
  PruneContainers(PruneContainers),
  DeleteNetwork(DeleteNetwork),
  PruneNetworks(PruneNetworks),
  DeleteImage(DeleteImage),
  PruneImages(PruneImages),
  DeleteVolume(DeleteVolume),
  PruneVolumes(PruneVolumes),
  PruneDockerBuilders(PruneDockerBuilders),
  PruneBuildx(PruneBuildx),
  PruneSystem(PruneSystem),

  // ==== SWARM ====
  RemoveSwarmNodes(RemoveSwarmNodes),
  RemoveSwarmStacks(RemoveSwarmStacks),
  RemoveSwarmServices(RemoveSwarmServices),
  RemoveSwarmConfigs(RemoveSwarmConfigs),
  RemoveSwarmSecrets(RemoveSwarmSecrets),

  // ==== MAINTENANCE ====
  ClearRepoCache(ClearRepoCache),
  BackupCoreDatabase(BackupCoreDatabase),
  GlobalAutoUpdate(GlobalAutoUpdate),
  RotateAllServerKeys(RotateAllServerKeys),
  RotateCoreKeys(RotateCoreKeys),
}

pub fn router() -> Router {
  Router::new()
    .route("/", post(handler))
    .route("/{variant}", post(variant_handler))
    .layer(middleware::from_fn(auth_request))
}

async fn variant_handler(
  user: Extension<User>,
  Path(Variant { variant }): Path<Variant>,
  Json(params): Json<serde_json::Value>,
) -> serror::Result<(TypedHeader<ContentType>, String)> {
  let req: ExecuteRequest = serde_json::from_value(json!({
    "type": variant,
    "params": params,
  }))?;
  handler(user, Json(req)).await
}

async fn handler(
  Extension(user): Extension<User>,
  Json(request): Json<ExecuteRequest>,
) -> serror::Result<(TypedHeader<ContentType>, String)> {
  let res = match inner_handler(request, user).await? {
    ExecutionResult::Single(update) => serde_json::to_string(&update)
      .context("Failed to serialize Update")?,
    ExecutionResult::Batch(res) => res,
  };
  Ok((TypedHeader(ContentType::json()), res))
}

#[typeshare(serialized_as = "Update")]
type BoxUpdate = Box<Update>;

pub enum ExecutionResult {
  Single(BoxUpdate),
  /// The batch contents will be pre serialized here
  Batch(String),
}

pub fn inner_handler(
  request: ExecuteRequest,
  user: User,
) -> Pin<
  Box<
    dyn std::future::Future<Output = anyhow::Result<ExecutionResult>>
      + Send,
  >,
> {
  Box::pin(async move {
    let task_id = Uuid::new_v4();

    // Need to validate no cancel is active before any update is created.
    // This ensures no double update created if Cancel is called more than once for the same request.
    build::validate_cancel_build(&request).await?;
    repo::validate_cancel_repo_build(&request).await?;

    let update = init_execution_update(&request, &user).await?;

    // This will be the case for the Batch exections,
    // they don't have their own updates.
    // The batch calls also call "inner_handler" themselves,
    // and in their case will spawn tasks, so that isn't necessary
    // here either.
    if update.operation == Operation::None {
      return Ok(ExecutionResult::Batch(
        task(task_id, request, user, update).await?,
      ));
    }

    // Spawn a task for the execution which continues
    // running after this method returns.
    let handle =
      tokio::spawn(task(task_id, request, user, update.clone()));

    // Spawns another task to monitor the first for failures,
    // and add the log to Update about it (which primary task can't do because it errored out)
    tokio::spawn({
      let update_id = update.id.clone();
      async move {
        let log = match handle.await {
          Ok(Err(e)) => {
            warn!("/execute request {task_id} task error: {e:#}",);
            Log::error("Task Error", format_serror(&e.into()))
          }
          Err(e) => {
            warn!("/execute request {task_id} spawn error: {e:?}",);
            Log::error("Spawn Error", format!("{e:#?}"))
          }
          _ => return,
        };
        let res = async {
          // Nothing to do if update was never actually created,
          // which is the case when the id is empty.
          if update_id.is_empty() {
            return Ok(());
          }
          let mut update =
            find_one_by_id(&db_client().updates, &update_id)
              .await
              .context("failed to query to db")?
              .context("no update exists with given id")?;
          update.logs.push(log);
          update.finalize();
          update_update(update).await
        }
        .await;

        if let Err(e) = res {
          warn!(
            "failed to update update with task error log | {e:#}"
          );
        }
      }
    });

    Ok(ExecutionResult::Single(update.into()))
  })
}

async fn task(
  id: Uuid,
  request: ExecuteRequest,
  user: User,
  update: Update,
) -> anyhow::Result<String> {
  let variant = request.extract_variant();

  info!(
    "/execute request {id} | {variant} | user: {}",
    user.username
  );

  let res =
    match request.resolve(&ExecuteArgs { user, update, id }).await {
      Err(e) => Err(e.error),
      Ok(JsonString::Err(e)) => Err(
        anyhow::Error::from(e)
          .context("failed to serialize response"),
      ),
      Ok(JsonString::Ok(res)) => Ok(res),
    };

  if let Err(e) = &res {
    warn!("/execute request {id} error: {e:#}");
  }

  res
}

trait BatchExecute {
  type Resource: KomodoResource;
  fn single_request(name: String) -> ExecuteRequest;
}

#[instrument("BatchExecute", skip(user))]
async fn batch_execute<E: BatchExecute>(
  pattern: &str,
  user: &User,
) -> anyhow::Result<BatchExecutionResponse> {
  let resources = list_full_for_user_using_pattern::<E::Resource>(
    pattern,
    Default::default(),
    user,
    PermissionLevel::Execute.into(),
    &[],
  )
  .await?;

  let futures = resources.into_iter().map(|resource| {
    let user = user.clone();
    async move {
      inner_handler(E::single_request(resource.name.clone()), user)
        .await
        .map(|r| {
          let ExecutionResult::Single(update) = r else {
            unreachable!()
          };
          update
        })
        .map_err(|e| BatchExecutionResponseItemErr {
          name: resource.name,
          error: e.into(),
        })
        .into()
    }
  });
  Ok(join_all(futures).await)
}
