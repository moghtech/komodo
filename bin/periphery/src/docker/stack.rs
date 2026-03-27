use anyhow::{Context, anyhow};
use command::run_komodo_standard_command;
use futures_util::{StreamExt, stream::FuturesOrdered};
use komodo_client::entities::{
  docker::{
    service::SwarmServiceListItem,
    stack::{
      SwarmStack, SwarmStackListItem, SwarmStackServiceListItem,
      SwarmStackTaskListItem,
    },
  },
  swarm::SwarmState,
};

use super::*;

impl DockerClient {
  pub async fn inspect_swarm_stack(
    &self,
    name: String,
  ) -> anyhow::Result<SwarmStack> {
    let (service_ids, swarm_tasks, tasks) = tokio::try_join!(
      list_swarm_stack_service_ids(&name),
      list_swarm_stack_tasks(&name),
      self.list_swarm_tasks(),
    )?;
    let services = self
      .list_swarm_services()
      .await?
      .into_iter()
      .filter(|service| {
        service
          .id
          .as_ref()
          .map(|long_id| {
            service_ids
              .iter()
              // These service ids are shortened coming from docker,
              // only need to check that long id starts with short id
              .any(|short_id| long_id.starts_with(short_id))
          })
          .unwrap_or_default()
      })
      .collect::<Vec<_>>();
    let task_ids = swarm_tasks
      .into_iter()
      .filter_map(|task| task.id)
      .collect::<Vec<_>>();
    let tasks = tasks
      .into_iter()
      .filter(|task| {
        task
          .id
          .as_ref()
          .map(|id| task_ids.contains(id))
          .unwrap_or_default()
      })
      .collect::<Vec<_>>();
    let state = state_from_services(&services);
    Ok(SwarmStack {
      name,
      state,
      services,
      tasks,
    })
  }
}

pub async fn list_swarm_stacks(
  services: &[SwarmServiceListItem],
) -> anyhow::Result<Vec<SwarmStackListItem>> {
  let res = run_komodo_standard_command(
    "List Swarm Stacks",
    None,
    "docker stack ls --format json",
  )
  .await;

  if !res.success {
    return Err(anyhow!("{}", res.combined()).context(
      "Failed to list swarm stacks using 'docker stack ls'",
    ));
  }

  // The output is in JSONL, need to convert to standard JSON vec.
  let mut stacks = serde_json::from_str::<Vec<SwarmStackListItem>>(
    &format!("[{}]", res.stdout.trim().replace('\n', ",")),
  )
  .context("Failed to parse 'docker stack ls' response from json")?
  // Attach state concurrently from tasks. Still include stack
  // if it fails, just with None state.
  .into_iter()
  .map(|mut stack| async move {
    let res = async {
      let service_ids =
        list_swarm_stack_service_ids(stack.name.as_ref()?)
          .await
          .ok()?;
      let services = services.iter().filter(|s| {
        let Some(id) = &s.id else {
          return false;
        };
        service_ids.iter().any(|sid| {
          // The service id may be short hash
          id.starts_with(sid)
        })
      });
      Some(state_from_services(services))
    }
    .await;
    if let Some(state) = res {
      stack.state = Some(state);
    }
    stack
  })
  .collect::<FuturesOrdered<_>>()
  .collect::<Vec<_>>()
  .await;

  stacks.sort_by(|a, b| {
    cmp_option(a.state, b.state)
      .then_with(|| cmp_option(a.name.as_ref(), b.name.as_ref()))
  });

  Ok(stacks)
}

pub async fn list_swarm_stack_service_ids(
  stack: &str,
) -> anyhow::Result<Vec<String>> {
  let res = run_komodo_standard_command(
    "List Swarm Stack Services",
    None,
    format!("docker stack services --format json {stack}"),
  )
  .await;

  if !res.success {
    return Err(anyhow!("{}", res.combined()).context(
      "Failed to list swarm stacks using 'docker stack services'",
    ));
  }

  // The output is in JSONL, need to convert to standard JSON vec.
  let ids = serde_json::from_str::<Vec<SwarmStackServiceListItem>>(
    &format!("[{}]", res.stdout.trim().replace('\n', ",")),
  )
  .context(
    "Failed to parse 'docker stack services' response from json",
  )?
  .into_iter()
  .filter_map(|service| service.id)
  .collect::<Vec<_>>();

  Ok(ids)
}

pub async fn list_swarm_stack_tasks(
  stack: &str,
) -> anyhow::Result<Vec<SwarmStackTaskListItem>> {
  let res = run_komodo_standard_command(
    "List Swarm Stack Tasks",
    None,
    format!("docker stack ps --format json --no-trunc {stack}"),
  )
  .await;

  if !res.success {
    return Err(anyhow!("{}", res.combined()).context(
      "Failed to list swarm stacks using 'docker stack ps'",
    ));
  }

  // The output is in JSONL, need to convert to standard JSON vec.
  let tasks = serde_json::from_str::<Vec<SwarmStackTaskListItem>>(
    &format!("[{}]", res.stdout.trim().replace('\n', ",")),
  )
  .context("Failed to parse 'docker stack ps' response from json")?;

  Ok(tasks)
}

pub fn state_from_services<'a>(
  services: impl IntoIterator<Item = &'a SwarmServiceListItem>,
) -> SwarmState {
  let mut state = SwarmState::Unknown;
  for service in services {
    if matches!(service.state, SwarmState::Unhealthy) {
      return SwarmState::Unhealthy;
    }
    if matches!(state, SwarmState::Unknown) {
      state = service.state;
      continue;
    }
    if state != service.state {
      return SwarmState::Unhealthy;
    }
  }
  if matches!(state, SwarmState::Unknown) {
    // This happens if services is empty ie down.
    SwarmState::Down
  } else {
    state
  }
}

fn cmp_option<T: Ord>(
  a: Option<T>,
  b: Option<T>,
) -> std::cmp::Ordering {
  match (a, b) {
    (Some(a), Some(b)) => a.cmp(&b),
    (Some(_), None) => std::cmp::Ordering::Less,
    (None, Some(_)) => std::cmp::Ordering::Greater,
    (None, None) => std::cmp::Ordering::Equal,
  }
}
