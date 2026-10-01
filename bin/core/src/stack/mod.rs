use anyhow::Context;
use komodo_client::entities::{
  SwarmOrServer, permission::PermissionLevelAndSpecifics,
  stack::Stack, user::User,
};
use regex::Regex;

use crate::{
  helpers::query::get_swarm_or_server,
  permission::get_check_permissions,
};

pub mod execute;
pub mod remote;
pub mod services;

pub async fn setup_stack_execution(
  stack: &str,
  user: &User,
  permissions: PermissionLevelAndSpecifics,
) -> anyhow::Result<(Stack, SwarmOrServer)> {
  let stack =
    get_check_permissions::<Stack>(stack, user, permissions).await?;

  let swarm_or_server = get_swarm_or_server(
    &stack.config.swarm_id,
    &stack.config.server_id,
  )
  .await?;

  Ok((stack, swarm_or_server))
}

/// `exact` should be set for the entries Komodo lists per replica
/// of a compose service with `deploy.replicas > 1`, whose `container_name`
/// already includes the replica number. Otherwise eg `web-1` would also
/// match the containers of replicas `web-10`, `web-11`, ...
pub fn compose_container_match_regex(
  container_name: &str,
  exact: bool,
) -> anyhow::Result<Regex> {
  let regex = if exact {
    format!("^{container_name}$")
  } else {
    format!("^{container_name}-?[0-9]*$")
  };
  Regex::new(&regex).with_context(|| {
    format!("failed to construct valid regex from {regex}")
  })
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn container_match_regex() {
    let regex =
      compose_container_match_regex("proj-web", false).unwrap();
    assert!(regex.is_match("proj-web"));
    assert!(regex.is_match("proj-web-1"));
    assert!(regex.is_match("proj-web-12"));

    // Replica entries already include the replica number.
    let regex =
      compose_container_match_regex("proj-web-1", true).unwrap();
    assert!(regex.is_match("proj-web-1"));
    assert!(!regex.is_match("proj-web-10"));
    assert!(!regex.is_match("proj-web-1-1"));
  }
}
