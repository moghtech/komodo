use anyhow::{Context, anyhow};
use command::run_komodo_standard_command;
use futures_util::{StreamExt, stream::FuturesOrdered};
use komodo_client::entities::{
  docker::stack::{
    SwarmStack, SwarmStackListItem, SwarmStackServiceListItem,
    SwarmStackTaskListItem,
  },
  swarm::SwarmState,
};

pub async fn inspect_swarm_stack(
  name: String,
) -> anyhow::Result<SwarmStack> {
  let (tasks, services) = tokio::try_join!(
    list_swarm_stack_tasks(&name),
    list_swarm_stack_services(&name)
  )?;
  let state = state_from_tasks(&tasks);
  Ok(SwarmStack {
    name,
    state,
    services,
    tasks,
  })
}

pub async fn list_swarm_stacks()
-> anyhow::Result<Vec<SwarmStackListItem>> {
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
      let tasks =
        list_swarm_stack_tasks(stack.name.as_ref()?).await.ok()?;
      Some(state_from_tasks(&tasks))
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

pub async fn list_swarm_stack_services(
  stack: &str,
) -> anyhow::Result<Vec<SwarmStackServiceListItem>> {
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
  let mut services =
    serde_json::from_str::<Vec<SwarmStackServiceListItem>>(&format!(
      "[{}]",
      res.stdout.trim().replace('\n', ",")
    ))
    .context(
      "Failed to parse 'docker stack services' response from json",
    )?;

  services.sort_by(|a, b| a.name.cmp(&b.name));

  Ok(services)
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
  let mut tasks =
    serde_json::from_str::<Vec<SwarmStackTaskListItem>>(&format!(
      "[{}]",
      res.stdout.trim().replace('\n', ",")
    ))
    .context(
      "Failed to parse 'docker stack ps' response from json",
    )?;

  tasks.sort_by(|a, b| {
    a.desired_state
      .cmp(&b.desired_state)
      .then_with(|| a.name.cmp(&b.name))
  });

  Ok(tasks)
}

pub fn state_from_tasks<'a>(
  tasks: impl IntoIterator<Item = &'a SwarmStackTaskListItem>,
) -> SwarmState {
  for task in tasks {
    let (Some(current), Some(desired)) =
      (&task.current_state, &task.desired_state)
    else {
      continue;
    };
    // CurrentState example: 'Running 44 minutes ago'.
    // Only want first "word"
    let Some(current) = current.split(" ").next() else {
      continue;
    };
    if current != desired {
      return SwarmState::Unhealthy;
    }
  }
  SwarmState::Healthy
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
