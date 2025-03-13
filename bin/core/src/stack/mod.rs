use anyhow::{Context, anyhow};
use komodo_client::entities::{
  permission::PermissionLevel,
  server::{Server, ServerState},
  stack::Stack,
  user::User,
};
use regex::Regex;

use crate::{helpers::query::get_server_with_state, resource};

pub mod execute;
pub mod remote;
pub mod services;

pub async fn get_stack_and_server(
  stack: &str,
  user: &User,
  permission_level: PermissionLevel,
  block_if_server_unreachable: bool,
) -> anyhow::Result<(Stack, Server)> {
  let stack = resource::get_check_permissions::<Stack>(
    stack,
    user,
    permission_level,
  )
  .await?;

  if stack.config.server_id.is_empty() {
    return Err(anyhow!("Stack has no server configured"));
  }

  let (server, status) =
    get_server_with_state(&stack.config.server_id).await?;
  if block_if_server_unreachable && status != ServerState::Ok {
    return Err(anyhow!(
      "cannot send action when server is unreachable or disabled"
    ));
  }

  Ok((stack, server))
}

pub fn compose_container_match_regex(
  container_name: &str,
) -> anyhow::Result<Regex> {
  let regex = format!("^{container_name}-?[0-9]*$");
  Regex::new(&regex).with_context(|| {
    format!("failed to construct valid regex from {regex}")
  })
}
