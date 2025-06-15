use anyhow::Context;
use axum::{Extension, Router, middleware, routing::post};
use komodo_client::{
  api::terminal::*,
  entities::{
    deployment::Deployment, permission::PermissionLevel,
    server::Server, stack::Stack, user::User,
  },
};
use serror::Json;
use uuid::Uuid;

use crate::{
  auth::auth_request, helpers::periphery_client,
  permission::get_check_permissions, resource::get,
  state::stack_status_cache,
};

pub fn router() -> Router {
  Router::new()
    .route("/execute", post(execute_terminal))
    .route("/execute/container", post(execute_container_exec))
    .route("/execute/deployment", post(execute_deployment_exec))
    .route("/execute/stack", post(execute_stack_exec))
    .layer(middleware::from_fn(auth_request))
}

// =================
//  ExecuteTerminal
// =================

async fn execute_terminal(
  Extension(user): Extension<User>,
  Json(request): Json<ExecuteTerminalBody>,
) -> serror::Result<axum::body::Body> {
  execute_terminal_inner(Uuid::new_v4(), request, user).await
}

#[instrument(
  name = "ExecuteTerminal",
  skip(user),
  fields(
    user_id = user.id,
  )
)]
async fn execute_terminal_inner(
  req_id: Uuid,
  ExecuteTerminalBody {
    server,
    terminal,
    command,
  }: ExecuteTerminalBody,
  user: User,
) -> serror::Result<axum::body::Body> {
  info!("/terminal/execute request | user: {}", user.username);

  let res = async {
    let server = get_check_permissions::<Server>(
      &server,
      &user,
      PermissionLevel::Read.terminal(),
    )
    .await?;

    let periphery = periphery_client(&server)?;

    let stream = periphery
      .execute_terminal(terminal, command)
      .await
      .context("Failed to execute command on periphery")?;

    anyhow::Ok(stream)
  }
  .await;

  let stream = match res {
    Ok(stream) => stream,
    Err(e) => {
      warn!("/terminal/execute request {req_id} error: {e:#}");
      return Err(e.into());
    }
  };

  Ok(axum::body::Body::from_stream(stream.into_line_stream()))
}

// ======================
//  ExecuteContainerExec
// ======================

async fn execute_container_exec(
  Extension(user): Extension<User>,
  Json(request): Json<ExecuteContainerExecBody>,
) -> serror::Result<axum::body::Body> {
  execute_container_exec_inner(Uuid::new_v4(), request, user).await
}

#[instrument(
  name = "ExecuteContainerExec",
  skip(user),
  fields(
    user_id = user.id,
  )
)]
async fn execute_container_exec_inner(
  req_id: Uuid,
  ExecuteContainerExecBody {
    server,
    container,
    shell,
    command,
  }: ExecuteContainerExecBody,
  user: User,
) -> serror::Result<axum::body::Body> {
  info!(
    "/terminal/execute/container request | user: {}",
    user.username
  );

  let res = async {
    let server = get_check_permissions::<Server>(
      &server,
      &user,
      PermissionLevel::Read.terminal(),
    )
    .await?;

    let periphery = periphery_client(&server)?;

    let stream = periphery
      .execute_container_exec(container, shell, command)
      .await
      .context(
        "Failed to execute container exec command on periphery",
      )?;

    anyhow::Ok(stream)
  }
  .await;

  let stream = match res {
    Ok(stream) => stream,
    Err(e) => {
      warn!(
        "/terminal/execute/container request {req_id} error: {e:#}"
      );
      return Err(e.into());
    }
  };

  Ok(axum::body::Body::from_stream(stream.into_line_stream()))
}

// =======================
//  ExecuteDeploymentExec
// =======================

async fn execute_deployment_exec(
  Extension(user): Extension<User>,
  Json(request): Json<ExecuteDeploymentExecBody>,
) -> serror::Result<axum::body::Body> {
  execute_deployment_exec_inner(Uuid::new_v4(), request, user).await
}

#[instrument(
  name = "ExecuteDeploymentExec",
  skip(user),
  fields(
    user_id = user.id,
  )
)]
async fn execute_deployment_exec_inner(
  req_id: Uuid,
  ExecuteDeploymentExecBody {
    deployment,
    shell,
    command,
  }: ExecuteDeploymentExecBody,
  user: User,
) -> serror::Result<axum::body::Body> {
  info!(
    "/terminal/execute/deployment request | user: {}",
    user.username
  );

  let res = async {
    let deployment = get_check_permissions::<Deployment>(
      &deployment,
      &user,
      PermissionLevel::Read.terminal(),
    )
    .await?;

    let server = get::<Server>(&deployment.config.server_id).await?;

    let periphery = periphery_client(&server)?;

    let stream = periphery
      .execute_container_exec(deployment.name, shell, command)
      .await
      .context(
        "Failed to execute container exec command on periphery",
      )?;

    anyhow::Ok(stream)
  }
  .await;

  let stream = match res {
    Ok(stream) => stream,
    Err(e) => {
      warn!(
        "/terminal/execute/deployment request {req_id} error: {e:#}"
      );
      return Err(e.into());
    }
  };

  Ok(axum::body::Body::from_stream(stream.into_line_stream()))
}

// ==================
//  ExecuteStackExec
// ==================

async fn execute_stack_exec(
  Extension(user): Extension<User>,
  Json(request): Json<ExecuteStackExecBody>,
) -> serror::Result<axum::body::Body> {
  execute_stack_exec_inner(Uuid::new_v4(), request, user).await
}

#[instrument(
  name = "ExecuteStackExec",
  skip(user),
  fields(
    user_id = user.id,
  )
)]
async fn execute_stack_exec_inner(
  req_id: Uuid,
  ExecuteStackExecBody {
    stack,
    service,
    shell,
    command,
  }: ExecuteStackExecBody,
  user: User,
) -> serror::Result<axum::body::Body> {
  info!("/terminal/execute/stack request | user: {}", user.username);

  let res = async {
    let stack = get_check_permissions::<Stack>(
      &stack,
      &user,
      PermissionLevel::Read.terminal(),
    )
    .await?;

    let server = get::<Server>(&stack.config.server_id).await?;

    let container = stack_status_cache()
      .get(&stack.id)
      .await
      .context("could not get stack status")?
      .curr
      .services
      .iter()
      .find(|s| s.service == service)
      .context("could not find service")?
      .container
      .as_ref()
      .context("could not find service container")?
      .name
      .clone();

    let periphery = periphery_client(&server)?;

    let stream = periphery
      .execute_container_exec(container, shell, command)
      .await
      .context(
        "Failed to execute container exec command on periphery",
      )?;

    anyhow::Ok(stream)
  }
  .await;

  let stream = match res {
    Ok(stream) => stream,
    Err(e) => {
      warn!("/terminal/execute/stack request {req_id} error: {e:#}");
      return Err(e.into());
    }
  };

  Ok(axum::body::Body::from_stream(stream.into_line_stream()))
}
