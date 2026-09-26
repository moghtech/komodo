use anyhow::Context;
use komodo_client::entities::{
  SwarmOrServer, permission::PermissionLevelAndSpecifics,
  stack::Stack, user::User,
};
use regex::Regex;

use crate::{
  helpers::query::get_swarm_or_server,
  permission::get_check_any_permissions,
};

pub mod execute;
pub mod remote;
pub mod services;

pub async fn setup_stack_execution(
  stack: &str,
  user: &User,
  permissions: PermissionLevelAndSpecifics,
) -> anyhow::Result<(Stack, SwarmOrServer)> {
  setup_stack_execution_any(
    stack,
    user,
    std::slice::from_ref(&permissions),
  )
  .await
}

/// Like [setup_stack_execution], but passes when the user
/// fulfills any one of `allowed_permissions`.
pub async fn setup_stack_execution_any(
  stack: &str,
  user: &User,
  allowed_permissions: &[PermissionLevelAndSpecifics],
) -> anyhow::Result<(Stack, SwarmOrServer)> {
  let stack = get_check_any_permissions::<Stack>(
    stack,
    user,
    allowed_permissions,
  )
  .await?;

  let swarm_or_server = get_swarm_or_server(
    &stack.config.swarm_id,
    &stack.config.server_id,
  )
  .await?;

  Ok((stack, swarm_or_server))
}

pub fn compose_container_match_regex(
  container_name: &str,
) -> anyhow::Result<Regex> {
  let regex = format!("^{container_name}-?[0-9]*$");
  Regex::new(&regex).with_context(|| {
    format!("failed to construct valid regex from {regex}")
  })
}
