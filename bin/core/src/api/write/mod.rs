use std::time::Instant;

use anyhow::Context;
use axum::{
  Extension, Router, extract::Path, middleware, routing::post, http::StatusCode,
};
use derive_variants::{EnumVariants, ExtractVariant};
use komodo_client::{api::write::*, entities::user::User};
use resolver_api::Resolve;
use response::Response;
use serde::{Deserialize, Serialize};
use serde_json::json;
use serror::{Json, AddStatusCodeError};
use typeshare::typeshare;
use uuid::Uuid;

use crate::auth::auth_request;

use super::Variant;

mod action;
mod alerter;
mod build;
mod builder;
mod deployment;
mod permissions;
mod procedure;
mod provider;
mod repo;
mod resource;
mod server;
mod service_user;
mod stack;
mod sync;
mod tag;
mod user;
mod user_group;
mod variable;

pub struct WriteArgs {
  pub user: User,
}

/// Helper function to handle resource creation errors with proper HTTP status codes
pub fn handle_resource_creation_error(e: anyhow::Error) -> serror::Error {
  let error_msg = e.to_string();
  
  // Check if this is a validation error and set appropriate status code
  if error_msg.contains("Resource with name") && error_msg.contains("already exists") {
    e.status_code(StatusCode::CONFLICT)
  } else if error_msg.contains("Must provide unique name for resource") {
    e.status_code(StatusCode::CONFLICT)
  } else if error_msg.contains("busy") {
    e.status_code(StatusCode::CONFLICT)
  } else if error_msg.contains("Valid ObjectIds cannot be used as names") 
         || error_msg.contains("Must provide non-empty name") 
         || error_msg.contains("User does not have permissions") {
    e.status_code(StatusCode::BAD_REQUEST)
  } else {
    e.into()
  }
}

#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EnumVariants,
)]
#[variant_derive(Debug)]
#[args(WriteArgs)]
#[response(Response)]
#[error(serror::Error)]
#[serde(tag = "type", content = "params")]
pub enum WriteRequest {
  // ==== USER ====
  CreateLocalUser(CreateLocalUser),
  UpdateUserUsername(UpdateUserUsername),
  UpdateUserPassword(UpdateUserPassword),
  DeleteUser(DeleteUser),

  // ==== SERVICE USER ====
  CreateServiceUser(CreateServiceUser),
  UpdateServiceUserDescription(UpdateServiceUserDescription),
  CreateApiKeyForServiceUser(CreateApiKeyForServiceUser),
  DeleteApiKeyForServiceUser(DeleteApiKeyForServiceUser),

  // ==== USER GROUP ====
  CreateUserGroup(CreateUserGroup),
  RenameUserGroup(RenameUserGroup),
  DeleteUserGroup(DeleteUserGroup),
  AddUserToUserGroup(AddUserToUserGroup),
  RemoveUserFromUserGroup(RemoveUserFromUserGroup),
  SetUsersInUserGroup(SetUsersInUserGroup),
  SetEveryoneUserGroup(SetEveryoneUserGroup),

  // ==== PERMISSIONS ====
  UpdateUserAdmin(UpdateUserAdmin),
  UpdateUserBasePermissions(UpdateUserBasePermissions),
  UpdatePermissionOnResourceType(UpdatePermissionOnResourceType),
  UpdatePermissionOnTarget(UpdatePermissionOnTarget),

  // ==== RESOURCE ====
  UpdateResourceMeta(UpdateResourceMeta),

  // ==== SERVER ====
  CreateServer(CreateServer),
  CopyServer(CopyServer),
  DeleteServer(DeleteServer),
  UpdateServer(UpdateServer),
  RenameServer(RenameServer),
  CreateNetwork(CreateNetwork),
  CreateTerminal(CreateTerminal),
  DeleteTerminal(DeleteTerminal),
  DeleteAllTerminals(DeleteAllTerminals),

  // ==== STACK ====
  CreateStack(CreateStack),
  CopyStack(CopyStack),
  DeleteStack(DeleteStack),
  UpdateStack(UpdateStack),
  RenameStack(RenameStack),
  WriteStackFileContents(WriteStackFileContents),
  RefreshStackCache(RefreshStackCache),
  CreateStackWebhook(CreateStackWebhook),
  DeleteStackWebhook(DeleteStackWebhook),

  // ==== DEPLOYMENT ====
  CreateDeployment(CreateDeployment),
  CopyDeployment(CopyDeployment),
  CreateDeploymentFromContainer(CreateDeploymentFromContainer),
  DeleteDeployment(DeleteDeployment),
  UpdateDeployment(UpdateDeployment),
  RenameDeployment(RenameDeployment),

  // ==== BUILD ====
  CreateBuild(CreateBuild),
  CopyBuild(CopyBuild),
  DeleteBuild(DeleteBuild),
  UpdateBuild(UpdateBuild),
  RenameBuild(RenameBuild),
  WriteBuildFileContents(WriteBuildFileContents),
  RefreshBuildCache(RefreshBuildCache),
  CreateBuildWebhook(CreateBuildWebhook),
  DeleteBuildWebhook(DeleteBuildWebhook),

  // ==== BUILDER ====
  CreateBuilder(CreateBuilder),
  CopyBuilder(CopyBuilder),
  DeleteBuilder(DeleteBuilder),
  UpdateBuilder(UpdateBuilder),
  RenameBuilder(RenameBuilder),

  // ==== REPO ====
  CreateRepo(CreateRepo),
  CopyRepo(CopyRepo),
  DeleteRepo(DeleteRepo),
  UpdateRepo(UpdateRepo),
  RenameRepo(RenameRepo),
  RefreshRepoCache(RefreshRepoCache),
  CreateRepoWebhook(CreateRepoWebhook),
  DeleteRepoWebhook(DeleteRepoWebhook),

  // ==== ALERTER ====
  CreateAlerter(CreateAlerter),
  CopyAlerter(CopyAlerter),
  DeleteAlerter(DeleteAlerter),
  UpdateAlerter(UpdateAlerter),
  RenameAlerter(RenameAlerter),

  // ==== PROCEDURE ====
  CreateProcedure(CreateProcedure),
  CopyProcedure(CopyProcedure),
  DeleteProcedure(DeleteProcedure),
  UpdateProcedure(UpdateProcedure),
  RenameProcedure(RenameProcedure),

  // ==== ACTION ====
  CreateAction(CreateAction),
  CopyAction(CopyAction),
  DeleteAction(DeleteAction),
  UpdateAction(UpdateAction),
  RenameAction(RenameAction),

  // ==== SYNC ====
  CreateResourceSync(CreateResourceSync),
  CopyResourceSync(CopyResourceSync),
  DeleteResourceSync(DeleteResourceSync),
  UpdateResourceSync(UpdateResourceSync),
  RenameResourceSync(RenameResourceSync),
  WriteSyncFileContents(WriteSyncFileContents),
  CommitSync(CommitSync),
  RefreshResourceSyncPending(RefreshResourceSyncPending),
  CreateSyncWebhook(CreateSyncWebhook),
  DeleteSyncWebhook(DeleteSyncWebhook),

  // ==== TAG ====
  CreateTag(CreateTag),
  DeleteTag(DeleteTag),
  RenameTag(RenameTag),
  UpdateTagColor(UpdateTagColor),

  // ==== VARIABLE ====
  CreateVariable(CreateVariable),
  UpdateVariableValue(UpdateVariableValue),
  UpdateVariableDescription(UpdateVariableDescription),
  UpdateVariableIsSecret(UpdateVariableIsSecret),
  DeleteVariable(DeleteVariable),

  // ==== PROVIDERS ====
  CreateGitProviderAccount(CreateGitProviderAccount),
  UpdateGitProviderAccount(UpdateGitProviderAccount),
  DeleteGitProviderAccount(DeleteGitProviderAccount),
  CreateDockerRegistryAccount(CreateDockerRegistryAccount),
  UpdateDockerRegistryAccount(UpdateDockerRegistryAccount),
  DeleteDockerRegistryAccount(DeleteDockerRegistryAccount),
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
) -> serror::Result<axum::response::Response> {
  let req: WriteRequest = serde_json::from_value(json!({
    "type": variant,
    "params": params,
  }))?;
  handler(user, Json(req)).await
}

async fn handler(
  Extension(user): Extension<User>,
  Json(request): Json<WriteRequest>,
) -> serror::Result<axum::response::Response> {
  let req_id = Uuid::new_v4();

  let res = tokio::spawn(task(req_id, request, user))
    .await
    .context("failure in spawned task");

  res?
}

#[instrument(
  name = "WriteRequest",
  skip(user, request),
  fields(
    user_id = user.id,
    request = format!("{:?}", request.extract_variant())
  )
)]
async fn task(
  req_id: Uuid,
  request: WriteRequest,
  user: User,
) -> serror::Result<axum::response::Response> {
  info!("/write request | user: {}", user.username);

  let timer = Instant::now();

  let res = request.resolve(&WriteArgs { user }).await;

  if let Err(e) = &res {
    warn!("/write request {req_id} error: {:#}", e.error);
  }

  let elapsed = timer.elapsed();
  debug!("/write request {req_id} | resolve time: {elapsed:?}");

  res.map(|res| res.0)
}
