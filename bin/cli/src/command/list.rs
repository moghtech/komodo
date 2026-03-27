use std::{cmp::Ordering, collections::HashMap};

use comfy_table::{Attribute, Cell, Color};
use futures_util::{FutureExt, try_join};
use komodo_client::{
  KomodoClient,
  api::read::{
    ListActions, ListAlerters, ListBuilders, ListBuilds,
    ListDeployments, ListProcedures, ListRepos, ListResourceSyncs,
    ListSchedules, ListServers, ListStacks, ListTags, ListTerminals,
  },
  entities::{
    ResourceTargetVariant,
    action::{ActionListItem, ActionListItemInfo, ActionState},
    alerter::{AlerterListItem, AlerterListItemInfo},
    build::{BuildListItem, BuildListItemInfo, BuildState},
    builder::{BuilderListItem, BuilderListItemInfo},
    config::cli::args::{
      self,
      list::{ListCommand, ResourceFilters},
    },
    deployment::{
      DeploymentListItem, DeploymentListItemInfo, DeploymentState,
    },
    procedure::{
      ProcedureListItem, ProcedureListItemInfo, ProcedureState,
    },
    repo::{RepoListItem, RepoListItemInfo, RepoState},
    resource::{ResourceListItem, ResourceQuery},
    resource_link,
    schedule::Schedule,
    server::{ServerListItem, ServerListItemInfo, ServerState},
    stack::{StackListItem, StackListItemInfo, StackState},
    sync::{
      ResourceSyncListItem, ResourceSyncListItemInfo,
      ResourceSyncState,
    },
    terminal::Terminal,
  },
};
use serde::Serialize;

use crate::{
  command::{
    PrintTable, format_timetamp, matches_wildcards, parse_wildcards,
    print_items,
  },
  config::cli_config,
};

pub async fn handle(list: &args::list::List) -> anyhow::Result<()> {
  match &list.command {
    None => list_all(list).await,
    Some(ListCommand::Servers(filters)) => {
      list_resources::<ServerListItem>(filters, false).await
    }
    Some(ListCommand::Stacks(filters)) => {
      list_resources::<StackListItem>(filters, false).await
    }
    Some(ListCommand::Deployments(filters)) => {
      list_resources::<DeploymentListItem>(filters, false).await
    }
    Some(ListCommand::Builds(filters)) => {
      list_resources::<BuildListItem>(filters, false).await
    }
    Some(ListCommand::Repos(filters)) => {
      list_resources::<RepoListItem>(filters, false).await
    }
    Some(ListCommand::Procedures(filters)) => {
      list_resources::<ProcedureListItem>(filters, false).await
    }
    Some(ListCommand::Actions(filters)) => {
      list_resources::<ActionListItem>(filters, false).await
    }
    Some(ListCommand::Syncs(filters)) => {
      list_resources::<ResourceSyncListItem>(filters, false).await
    }
    Some(ListCommand::Terminals(filters)) => {
      list_terminals(filters).await
    }
    Some(ListCommand::Schedules(filters)) => {
      list_schedules(filters).await
    }
    Some(ListCommand::Builders(filters)) => {
      list_resources::<BuilderListItem>(filters, false).await
    }
    Some(ListCommand::Alerters(filters)) => {
      list_resources::<AlerterListItem>(filters, false).await
    }
  }
}

/// Includes all resources besides builds and alerters.
async fn list_all(list: &args::list::List) -> anyhow::Result<()> {
  let filters: ResourceFilters = list.clone().into();
  let client = super::komodo_client().await?;
  let (
    tags,
    mut servers,
    mut stacks,
    mut deployments,
    mut builds,
    mut repos,
    mut procedures,
    mut actions,
    mut syncs,
  ) = try_join!(
    client.read(ListTags::default()).map(|res| res.map(|res| res
      .into_iter()
      .map(|t| (t.id, t.name))
      .collect::<HashMap<_, _>>())),
    ServerListItem::list(client, &filters, true),
    StackListItem::list(client, &filters, true),
    DeploymentListItem::list(client, &filters, true),
    BuildListItem::list(client, &filters, true),
    RepoListItem::list(client, &filters, true),
    ProcedureListItem::list(client, &filters, true),
    ActionListItem::list(client, &filters, true),
    ResourceSyncListItem::list(client, &filters, true),
  )?;

  if !servers.is_empty() {
    fix_tags(&mut servers, &tags);
    print_items(servers, filters.format, list.links)?;
    println!();
  }

  if !stacks.is_empty() {
    fix_tags(&mut stacks, &tags);
    print_items(stacks, filters.format, list.links)?;
    println!();
  }

  if !deployments.is_empty() {
    fix_tags(&mut deployments, &tags);
    print_items(deployments, filters.format, list.links)?;
    println!();
  }

  if !builds.is_empty() {
    fix_tags(&mut builds, &tags);
    print_items(builds, filters.format, list.links)?;
    println!();
  }

  if !repos.is_empty() {
    fix_tags(&mut repos, &tags);
    print_items(repos, filters.format, list.links)?;
    println!();
  }

  if !procedures.is_empty() {
    fix_tags(&mut procedures, &tags);
    print_items(procedures, filters.format, list.links)?;
    println!();
  }

  if !actions.is_empty() {
    fix_tags(&mut actions, &tags);
    print_items(actions, filters.format, list.links)?;
    println!();
  }

  if !syncs.is_empty() {
    fix_tags(&mut syncs, &tags);
    print_items(syncs, filters.format, list.links)?;
    println!();
  }

  Ok(())
}

async fn list_resources<T>(
  filters: &ResourceFilters,
  minimal: bool,
) -> anyhow::Result<()>
where
  T: ListResources,
  ResourceListItem<T::Info>: PrintTable + Serialize,
{
  let client = crate::command::komodo_client().await?;
  let (mut resources, tags) = tokio::try_join!(
    T::list(client, filters, minimal),
    client.read(ListTags::default()).map(|res| res.map(|res| res
      .into_iter()
      .map(|t| (t.id, t.name))
      .collect::<HashMap<_, _>>()))
  )?;
  fix_tags(&mut resources, &tags);
  if !resources.is_empty() {
    print_items(resources, filters.format, filters.links)?;
  }
  Ok(())
}

async fn list_terminals(
  filters: &ResourceFilters,
) -> anyhow::Result<()> {
  let client = crate::command::komodo_client().await?;
  // let query = ResourceQuery::builder()
  //   .tags(filters.tags.clone())
  //   .templates(TemplatesQueryBehavior::Exclude)
  //   .build();
  let terminals = client
    .read(ListTerminals {
      target: None,
      use_names: true,
    })
    .await?;
  if !terminals.is_empty() {
    print_items(terminals, filters.format, filters.links)?;
  }
  Ok(())
}

async fn list_schedules(
  filters: &ResourceFilters,
) -> anyhow::Result<()> {
  let client = crate::command::komodo_client().await?;
  let (mut schedules, tags) = tokio::try_join!(
    client
      .read(ListSchedules {
        tags: filters.tags.clone(),
        tag_behavior: Default::default(),
      })
      .map(|res| res.map(|res| res
        .into_iter()
        .filter(|s| s.next_scheduled_run.is_some())
        .collect::<Vec<_>>())),
    client.read(ListTags::default()).map(|res| res.map(|res| res
      .into_iter()
      .map(|t| (t.id, t.name))
      .collect::<HashMap<_, _>>()))
  )?;
  schedules.iter_mut().for_each(|resource| {
    resource.tags.iter_mut().for_each(|id| {
      let Some(name) = tags.get(id) else {
        *id = String::new();
        return;
      };
      id.clone_from(name);
    });
  });
  schedules.sort_by(|a, b| {
    match (a.next_scheduled_run, b.next_scheduled_run) {
      (Some(_), None) => return Ordering::Less,
      (None, Some(_)) => return Ordering::Greater,
      (Some(a), Some(b)) => return a.cmp(&b),
      (None, None) => {}
    }
    a.name.cmp(&b.name).then(a.enabled.cmp(&b.enabled))
  });
  if !schedules.is_empty() {
    print_items(schedules, filters.format, filters.links)?;
  }
  Ok(())
}

fn fix_tags<T>(
  resources: &mut [ResourceListItem<T>],
  tags: &HashMap<String, String>,
) {
  resources.iter_mut().for_each(|resource| {
    resource.tags.iter_mut().for_each(|id| {
      let Some(name) = tags.get(id) else {
        *id = String::new();
        return;
      };
      id.clone_from(name);
    });
  });
}

trait ListResources: Sized
where
  ResourceListItem<Self::Info>: PrintTable,
{
  type Info;
  async fn list(
    client: &KomodoClient,
    filters: &ResourceFilters,
    // For use with root `km ls`
    minimal: bool,
  ) -> anyhow::Result<Vec<ResourceListItem<Self::Info>>>;
}

// LIST

impl ListResources for ServerListItem {
  type Info = ServerListItemInfo;
  async fn list(
    client: &KomodoClient,
    filters: &ResourceFilters,
    _minimal: bool,
  ) -> anyhow::Result<Vec<Self>> {
    let servers = client
      .read(ListServers {
        query: ResourceQuery::builder()
          .tags(filters.tags.clone())
          // .tag_behavior(TagQueryBehavior::Any)
          .templates(filters.templates)
          .build(),
      })
      .await?;
    let names = parse_wildcards(&filters.names);
    let server_wildcards = parse_wildcards(&filters.servers);
    let mut servers = servers
      .into_iter()
      .filter(|server| {
        let state_check = if filters.all {
          true
        } else if filters.down {
          !matches!(server.info.state, ServerState::Ok)
        } else if filters.in_progress {
          false
        } else {
          matches!(server.info.state, ServerState::Ok)
        };
        let name_items = &[server.name.as_str()];
        state_check
          && matches_wildcards(&names, name_items)
          && matches_wildcards(&server_wildcards, name_items)
      })
      .collect::<Vec<_>>();
    servers.sort_by(|a, b| {
      a.info.state.cmp(&b.info.state).then(a.name.cmp(&b.name))
    });
    Ok(servers)
  }
}

impl ListResources for StackListItem {
  type Info = StackListItemInfo;
  async fn list(
    client: &KomodoClient,
    filters: &ResourceFilters,
    _minimal: bool,
  ) -> anyhow::Result<Vec<Self>> {
    let (servers, mut stacks) = tokio::try_join!(
      client
        .read(ListServers {
          query: ResourceQuery::builder().build(),
        })
        .map(|res| res.map(|res| res
          .into_iter()
          .map(|s| (s.id.clone(), s))
          .collect::<HashMap<_, _>>())),
      client.read(ListStacks {
        query: ResourceQuery::builder()
          .tags(filters.tags.clone())
          // .tag_behavior(TagQueryBehavior::Any)
          .templates(filters.templates)
          .build(),
      })
    )?;
    stacks.iter_mut().for_each(|stack| {
      if stack.info.server_id.is_empty() {
        return;
      }
      let Some(server) = servers.get(&stack.info.server_id) else {
        return;
      };
      stack.info.server_id.clone_from(&server.name);
    });
    let names = parse_wildcards(&filters.names);
    let servers = parse_wildcards(&filters.servers);
    let mut stacks = stacks
      .into_iter()
      .filter(|stack| {
        let state_check = if filters.all {
          true
        } else if filters.down {
          !matches!(
            stack.info.state,
            StackState::Running | StackState::Deploying
          )
        } else if filters.in_progress {
          matches!(stack.info.state, StackState::Deploying)
        } else {
          matches!(
            stack.info.state,
            StackState::Running | StackState::Deploying
          )
        };
        state_check
          && matches_wildcards(&names, &[stack.name.as_str()])
          && matches_wildcards(
            &servers,
            &[stack.info.server_id.as_str()],
          )
      })
      .collect::<Vec<_>>();
    stacks.sort_by(|a, b| {
      a.info
        .state
        .cmp(&b.info.state)
        .then(a.name.cmp(&b.name))
        .then(a.info.server_id.cmp(&b.info.server_id))
    });
    Ok(stacks)
  }
}

impl ListResources for DeploymentListItem {
  type Info = DeploymentListItemInfo;
  async fn list(
    client: &KomodoClient,
    filters: &ResourceFilters,
    _minimal: bool,
  ) -> anyhow::Result<Vec<Self>> {
    let (servers, mut deployments) = tokio::try_join!(
      client
        .read(ListServers {
          query: ResourceQuery::builder().build(),
        })
        .map(|res| res.map(|res| res
          .into_iter()
          .map(|s| (s.id.clone(), s))
          .collect::<HashMap<_, _>>())),
      client.read(ListDeployments {
        query: ResourceQuery::builder()
          .tags(filters.tags.clone())
          // .tag_behavior(TagQueryBehavior::Any)
          .templates(filters.templates)
          .build(),
      })
    )?;
    deployments.iter_mut().for_each(|deployment| {
      if deployment.info.server_id.is_empty() {
        return;
      }
      let Some(server) = servers.get(&deployment.info.server_id)
      else {
        return;
      };
      deployment.info.server_id.clone_from(&server.name);
    });
    let names = parse_wildcards(&filters.names);
    let servers = parse_wildcards(&filters.servers);
    let mut deployments = deployments
      .into_iter()
      .filter(|deployment| {
        let state_check = if filters.all {
          true
        } else if filters.down {
          !matches!(
            deployment.info.state,
            DeploymentState::Running | DeploymentState::Deploying
          )
        } else if filters.in_progress {
          matches!(deployment.info.state, DeploymentState::Deploying)
        } else {
          matches!(
            deployment.info.state,
            DeploymentState::Running | DeploymentState::Deploying
          )
        };
        state_check
          && matches_wildcards(&names, &[deployment.name.as_str()])
          && matches_wildcards(
            &servers,
            &[deployment.info.server_id.as_str()],
          )
      })
      .collect::<Vec<_>>();
    deployments.sort_by(|a, b| {
      a.info
        .state
        .cmp(&b.info.state)
        .then(a.name.cmp(&b.name))
        .then(a.info.server_id.cmp(&b.info.server_id))
    });
    Ok(deployments)
  }
}

impl ListResources for BuildListItem {
  type Info = BuildListItemInfo;
  async fn list(
    client: &KomodoClient,
    filters: &ResourceFilters,
    minimal: bool,
  ) -> anyhow::Result<Vec<Self>> {
    let (builders, mut builds) = tokio::try_join!(
      client
        .read(ListBuilders {
          query: ResourceQuery::builder().build(),
        })
        .map(|res| res.map(|res| res
          .into_iter()
          .map(|s| (s.id.clone(), s))
          .collect::<HashMap<_, _>>())),
      client.read(ListBuilds {
        query: ResourceQuery::builder()
          .tags(filters.tags.clone())
          // .tag_behavior(TagQueryBehavior::Any)
          .templates(filters.templates)
          .build(),
      })
    )?;
    builds.iter_mut().for_each(|build| {
      if build.info.builder_id.is_empty() {
        return;
      }
      let Some(builder) = builders.get(&build.info.builder_id) else {
        return;
      };
      build.info.builder_id.clone_from(&builder.name);
    });
    let names = parse_wildcards(&filters.names);
    let builders = parse_wildcards(&filters.builders);
    let mut builds = builds
      .into_iter()
      .filter(|build| {
        let state_check = if filters.all {
          true
        } else if filters.down {
          matches!(
            build.info.state,
            BuildState::Failed | BuildState::Unknown
          )
        } else if minimal || filters.in_progress {
          matches!(build.info.state, BuildState::Building)
        } else {
          true
        };
        state_check
          && matches_wildcards(&names, &[build.name.as_str()])
          && matches_wildcards(
            &builders,
            &[build.info.builder_id.as_str()],
          )
      })
      .collect::<Vec<_>>();
    builds.sort_by(|a, b| {
      a.name
        .cmp(&b.name)
        .then(a.info.builder_id.cmp(&b.info.builder_id))
        .then(a.info.state.cmp(&b.info.state))
    });
    Ok(builds)
  }
}

impl ListResources for RepoListItem {
  type Info = RepoListItemInfo;
  async fn list(
    client: &KomodoClient,
    filters: &ResourceFilters,
    minimal: bool,
  ) -> anyhow::Result<Vec<Self>> {
    let names = parse_wildcards(&filters.names);
    let mut repos = client
      .read(ListRepos {
        query: ResourceQuery::builder()
          .tags(filters.tags.clone())
          // .tag_behavior(TagQueryBehavior::Any)
          .templates(filters.templates)
          .build(),
      })
      .await?
      .into_iter()
      .filter(|repo| {
        let state_check = if filters.all {
          true
        } else if filters.down {
          matches!(
            repo.info.state,
            RepoState::Failed | RepoState::Unknown
          )
        } else if minimal || filters.in_progress {
          matches!(
            repo.info.state,
            RepoState::Building | RepoState::Cloning
          )
        } else {
          true
        };
        state_check
          && matches_wildcards(&names, &[repo.name.as_str()])
      })
      .collect::<Vec<_>>();
    repos.sort_by(|a, b| {
      a.name
        .cmp(&b.name)
        .then(a.info.server_id.cmp(&b.info.server_id))
        .then(a.info.builder_id.cmp(&b.info.builder_id))
    });
    Ok(repos)
  }
}

impl ListResources for ProcedureListItem {
  type Info = ProcedureListItemInfo;
  async fn list(
    client: &KomodoClient,
    filters: &ResourceFilters,
    minimal: bool,
  ) -> anyhow::Result<Vec<Self>> {
    let names = parse_wildcards(&filters.names);
    let mut procedures = client
      .read(ListProcedures {
        query: ResourceQuery::builder()
          .tags(filters.tags.clone())
          // .tag_behavior(TagQueryBehavior::Any)
          .templates(filters.templates)
          .build(),
      })
      .await?
      .into_iter()
      .filter(|procedure| {
        let state_check = if filters.all {
          true
        } else if filters.down {
          matches!(
            procedure.info.state,
            ProcedureState::Failed | ProcedureState::Unknown
          )
        } else if minimal || filters.in_progress {
          matches!(procedure.info.state, ProcedureState::Running)
        } else {
          true
        };
        state_check
          && matches_wildcards(&names, &[procedure.name.as_str()])
      })
      .collect::<Vec<_>>();
    procedures.sort_by(|a, b| {
      match (a.info.next_scheduled_run, b.info.next_scheduled_run) {
        (Some(_), None) => return Ordering::Less,
        (None, Some(_)) => return Ordering::Greater,
        (Some(a), Some(b)) => return a.cmp(&b),
        (None, None) => {}
      }
      a.name.cmp(&b.name).then(a.info.state.cmp(&b.info.state))
    });
    Ok(procedures)
  }
}

impl ListResources for ActionListItem {
  type Info = ActionListItemInfo;
  async fn list(
    client: &KomodoClient,
    filters: &ResourceFilters,
    minimal: bool,
  ) -> anyhow::Result<Vec<Self>> {
    let names = parse_wildcards(&filters.names);
    let mut actions = client
      .read(ListActions {
        query: ResourceQuery::builder()
          .tags(filters.tags.clone())
          // .tag_behavior(TagQueryBehavior::Any)
          .templates(filters.templates)
          .build(),
      })
      .await?
      .into_iter()
      .filter(|action| {
        let state_check = if filters.all {
          true
        } else if filters.down {
          matches!(
            action.info.state,
            ActionState::Failed | ActionState::Unknown
          )
        } else if minimal || filters.in_progress {
          matches!(action.info.state, ActionState::Running)
        } else {
          true
        };
        state_check
          && matches_wildcards(&names, &[action.name.as_str()])
      })
      .collect::<Vec<_>>();
    actions.sort_by(|a, b| {
      match (a.info.next_scheduled_run, b.info.next_scheduled_run) {
        (Some(_), None) => return Ordering::Less,
        (None, Some(_)) => return Ordering::Greater,
        (Some(a), Some(b)) => return a.cmp(&b),
        (None, None) => {}
      }
      a.name.cmp(&b.name).then(a.info.state.cmp(&b.info.state))
    });
    Ok(actions)
  }
}

impl ListResources for ResourceSyncListItem {
  type Info = ResourceSyncListItemInfo;
  async fn list(
    client: &KomodoClient,
    filters: &ResourceFilters,
    minimal: bool,
  ) -> anyhow::Result<Vec<Self>> {
    let names = parse_wildcards(&filters.names);
    let mut syncs = client
      .read(ListResourceSyncs {
        query: ResourceQuery::builder()
          .tags(filters.tags.clone())
          // .tag_behavior(TagQueryBehavior::Any)
          .templates(filters.templates)
          .build(),
      })
      .await?
      .into_iter()
      .filter(|sync| {
        let state_check = if filters.all {
          true
        } else if filters.down {
          matches!(
            sync.info.state,
            ResourceSyncState::Failed | ResourceSyncState::Unknown
          )
        } else if minimal || filters.in_progress {
          matches!(
            sync.info.state,
            ResourceSyncState::Syncing | ResourceSyncState::Pending
          )
        } else {
          true
        };
        state_check
          && matches_wildcards(&names, &[sync.name.as_str()])
      })
      .collect::<Vec<_>>();
    syncs.sort_by(|a, b| {
      a.name.cmp(&b.name).then(a.info.state.cmp(&b.info.state))
    });
    Ok(syncs)
  }
}

impl ListResources for BuilderListItem {
  type Info = BuilderListItemInfo;
  async fn list(
    client: &KomodoClient,
    filters: &ResourceFilters,
    minimal: bool,
  ) -> anyhow::Result<Vec<Self>> {
    let names = parse_wildcards(&filters.names);
    let mut builders = client
      .read(ListBuilders {
        query: ResourceQuery::builder()
          .tags(filters.tags.clone())
          // .tag_behavior(TagQueryBehavior::Any)
          .templates(filters.templates)
          .build(),
      })
      .await?
      .into_iter()
      .filter(|builder| {
        (!minimal || filters.all)
          && matches_wildcards(&names, &[builder.name.as_str()])
      })
      .collect::<Vec<_>>();
    builders.sort_by(|a, b| {
      a.name
        .cmp(&b.name)
        .then(a.info.builder_type.cmp(&b.info.builder_type))
    });
    Ok(builders)
  }
}

impl ListResources for AlerterListItem {
  type Info = AlerterListItemInfo;
  async fn list(
    client: &KomodoClient,
    filters: &ResourceFilters,
    minimal: bool,
  ) -> anyhow::Result<Vec<Self>> {
    let names = parse_wildcards(&filters.names);
    let mut syncs = client
      .read(ListAlerters {
        query: ResourceQuery::builder()
          .tags(filters.tags.clone())
          // .tag_behavior(TagQueryBehavior::Any)
          .templates(filters.templates)
          .build(),
      })
      .await?
      .into_iter()
      .filter(|sync| {
        (!minimal || filters.all)
          && matches_wildcards(&names, &[sync.name.as_str()])
      })
      .collect::<Vec<_>>();
    syncs.sort_by(|a, b| {
      a.info
        .enabled
        .cmp(&b.info.enabled)
        .then(a.name.cmp(&b.name))
        .then(a.info.endpoint_type.cmp(&b.info.endpoint_type))
    });
    Ok(syncs)
  }
}

// TABLE

impl PrintTable for ResourceListItem<ServerListItemInfo> {
  fn header(links: bool) -> &'static [&'static str] {
    if links {
      &["Server", "State", "Address", "Tags", "Link"]
    } else {
      &["Server", "State", "Address", "Tags"]
    }
  }
  fn row(self, links: bool) -> Vec<Cell> {
    let color = match self.info.state {
      ServerState::Ok => Color::Green,
      ServerState::NotOk => Color::Red,
      ServerState::Disabled => Color::Blue,
    };
    let mut res = vec![
      Cell::new(self.name).add_attribute(Attribute::Bold),
      Cell::new(self.info.state.to_string())
        .fg(color)
        .add_attribute(Attribute::Bold),
      Cell::new(self.info.address.as_deref().unwrap_or("inbound")),
      Cell::new(self.tags.join(", ")),
    ];
    if links {
      res.push(Cell::new(resource_link(
        &cli_config().host,
        ResourceTargetVariant::Server,
        &self.id,
      )))
    }
    res
  }
}

impl PrintTable for ResourceListItem<StackListItemInfo> {
  fn header(links: bool) -> &'static [&'static str] {
    if links {
      &["Stack", "State", "Server", "Tags", "Link"]
    } else {
      &["Stack", "State", "Server", "Tags"]
    }
  }
  fn row(self, links: bool) -> Vec<comfy_table::Cell> {
    let color = match self.info.state {
      StackState::Down => Color::Blue,
      StackState::Running => Color::Green,
      StackState::Deploying => Color::DarkYellow,
      StackState::Paused => Color::DarkYellow,
      StackState::Unknown => Color::Magenta,
      _ => Color::Red,
    };
    // let source = if self.info.files_on_host {
    //   "On Host"
    // } else if !self.info.repo.is_empty() {
    //   self.info.repo_link.as_str()
    // } else {
    //   "UI Defined"
    // };
    let mut res = vec![
      Cell::new(self.name).add_attribute(Attribute::Bold),
      Cell::new(self.info.state.to_string())
        .fg(color)
        .add_attribute(Attribute::Bold),
      Cell::new(self.info.server_id),
      // Cell::new(source),
      Cell::new(self.tags.join(", ")),
    ];
    if links {
      res.push(Cell::new(resource_link(
        &cli_config().host,
        ResourceTargetVariant::Stack,
        &self.id,
      )))
    }
    res
  }
}

impl PrintTable for ResourceListItem<DeploymentListItemInfo> {
  fn header(links: bool) -> &'static [&'static str] {
    if links {
      &["Deployment", "State", "Server", "Tags", "Link"]
    } else {
      &["Deployment", "State", "Server", "Tags"]
    }
  }
  fn row(self, links: bool) -> Vec<comfy_table::Cell> {
    let color = match self.info.state {
      DeploymentState::NotDeployed => Color::Blue,
      DeploymentState::Running => Color::Green,
      DeploymentState::Deploying => Color::DarkYellow,
      DeploymentState::Paused => Color::DarkYellow,
      DeploymentState::Unknown => Color::Magenta,
      _ => Color::Red,
    };
    let mut res = vec![
      Cell::new(self.name).add_attribute(Attribute::Bold),
      Cell::new(self.info.state.to_string())
        .fg(color)
        .add_attribute(Attribute::Bold),
      Cell::new(self.info.server_id),
      Cell::new(self.tags.join(", ")),
    ];
    if links {
      res.push(Cell::new(resource_link(
        &cli_config().host,
        ResourceTargetVariant::Deployment,
        &self.id,
      )))
    }
    res
  }
}

impl PrintTable for ResourceListItem<BuildListItemInfo> {
  fn header(links: bool) -> &'static [&'static str] {
    if links {
      &["Build", "State", "Builder", "Tags", "Link"]
    } else {
      &["Build", "State", "Builder", "Tags"]
    }
  }
  fn row(self, links: bool) -> Vec<comfy_table::Cell> {
    let color = match self.info.state {
      BuildState::Ok => Color::Green,
      BuildState::Building => Color::DarkYellow,
      BuildState::Unknown => Color::Magenta,
      BuildState::Failed => Color::Red,
    };
    let mut res = vec![
      Cell::new(self.name).add_attribute(Attribute::Bold),
      Cell::new(self.info.state.to_string())
        .fg(color)
        .add_attribute(Attribute::Bold),
      Cell::new(self.info.builder_id),
      Cell::new(self.tags.join(", ")),
    ];
    if links {
      res.push(Cell::new(resource_link(
        &cli_config().host,
        ResourceTargetVariant::Build,
        &self.id,
      )));
    }
    res
  }
}

impl PrintTable for ResourceListItem<RepoListItemInfo> {
  fn header(links: bool) -> &'static [&'static str] {
    if links {
      &["Repo", "State", "Link", "Tags", "Link"]
    } else {
      &["Repo", "State", "Link", "Tags"]
    }
  }
  fn row(self, links: bool) -> Vec<comfy_table::Cell> {
    let color = match self.info.state {
      RepoState::Ok => Color::Green,
      RepoState::Building
      | RepoState::Cloning
      | RepoState::Pulling => Color::DarkYellow,
      RepoState::Unknown => Color::Magenta,
      RepoState::Failed => Color::Red,
    };
    let mut res = vec![
      Cell::new(self.name).add_attribute(Attribute::Bold),
      Cell::new(self.info.state.to_string())
        .fg(color)
        .add_attribute(Attribute::Bold),
      Cell::new(self.info.repo_link),
      Cell::new(self.tags.join(", ")),
    ];
    if links {
      res.push(Cell::new(resource_link(
        &cli_config().host,
        ResourceTargetVariant::Repo,
        &self.id,
      )))
    }
    res
  }
}

impl PrintTable for ResourceListItem<ProcedureListItemInfo> {
  fn header(links: bool) -> &'static [&'static str] {
    if links {
      &["Procedure", "State", "Next Run", "Tags", "Link"]
    } else {
      &["Procedure", "State", "Next Run", "Tags"]
    }
  }
  fn row(self, links: bool) -> Vec<comfy_table::Cell> {
    let color = match self.info.state {
      ProcedureState::Ok => Color::Green,
      ProcedureState::Running => Color::DarkYellow,
      ProcedureState::Unknown => Color::Magenta,
      ProcedureState::Failed => Color::Red,
    };
    let next_run = if let Some(ts) = self.info.next_scheduled_run {
      Cell::new(
        format_timetamp(ts)
          .unwrap_or(String::from("Invalid next ts")),
      )
      .add_attribute(Attribute::Bold)
    } else {
      Cell::new(String::from("None"))
    };
    let mut res = vec![
      Cell::new(self.name).add_attribute(Attribute::Bold),
      Cell::new(self.info.state.to_string())
        .fg(color)
        .add_attribute(Attribute::Bold),
      next_run,
      Cell::new(self.tags.join(", ")),
    ];
    if links {
      res.push(Cell::new(resource_link(
        &cli_config().host,
        ResourceTargetVariant::Procedure,
        &self.id,
      )))
    }
    res
  }
}

impl PrintTable for ResourceListItem<ActionListItemInfo> {
  fn header(links: bool) -> &'static [&'static str] {
    if links {
      &["Action", "State", "Next Run", "Tags", "Link"]
    } else {
      &["Action", "State", "Next Run", "Tags"]
    }
  }
  fn row(self, links: bool) -> Vec<comfy_table::Cell> {
    let color = match self.info.state {
      ActionState::Ok => Color::Green,
      ActionState::Running => Color::DarkYellow,
      ActionState::Unknown => Color::Magenta,
      ActionState::Failed => Color::Red,
    };
    let next_run = if let Some(ts) = self.info.next_scheduled_run {
      Cell::new(
        format_timetamp(ts)
          .unwrap_or(String::from("Invalid next ts")),
      )
      .add_attribute(Attribute::Bold)
    } else {
      Cell::new(String::from("None"))
    };
    let mut res = vec![
      Cell::new(self.name).add_attribute(Attribute::Bold),
      Cell::new(self.info.state.to_string())
        .fg(color)
        .add_attribute(Attribute::Bold),
      next_run,
      Cell::new(self.tags.join(", ")),
    ];
    if links {
      res.push(Cell::new(resource_link(
        &cli_config().host,
        ResourceTargetVariant::Action,
        &self.id,
      )));
    }
    res
  }
}

impl PrintTable for ResourceListItem<ResourceSyncListItemInfo> {
  fn header(links: bool) -> &'static [&'static str] {
    if links {
      &["Sync", "State", "Tags", "Link"]
    } else {
      &["Sync", "State", "Tags"]
    }
  }
  fn row(self, links: bool) -> Vec<comfy_table::Cell> {
    let color = match self.info.state {
      ResourceSyncState::Ok => Color::Green,
      ResourceSyncState::Pending | ResourceSyncState::Syncing => {
        Color::DarkYellow
      }
      ResourceSyncState::Unknown => Color::Magenta,
      ResourceSyncState::Failed => Color::Red,
    };
    let mut res = vec![
      Cell::new(self.name).add_attribute(Attribute::Bold),
      Cell::new(self.info.state.to_string())
        .fg(color)
        .add_attribute(Attribute::Bold),
      Cell::new(self.tags.join(", ")),
    ];
    if links {
      res.push(Cell::new(resource_link(
        &cli_config().host,
        ResourceTargetVariant::ResourceSync,
        &self.id,
      )))
    }
    res
  }
}

impl PrintTable for ResourceListItem<BuilderListItemInfo> {
  fn header(links: bool) -> &'static [&'static str] {
    if links {
      &["Builder", "Type", "Tags", "Link"]
    } else {
      &["Builder", "Type", "Tags"]
    }
  }
  fn row(self, links: bool) -> Vec<comfy_table::Cell> {
    let mut res = vec![
      Cell::new(self.name).add_attribute(Attribute::Bold),
      Cell::new(self.info.builder_type),
      Cell::new(self.tags.join(", ")),
    ];
    if links {
      res.push(Cell::new(resource_link(
        &cli_config().host,
        ResourceTargetVariant::Builder,
        &self.id,
      )));
    }
    res
  }
}

impl PrintTable for ResourceListItem<AlerterListItemInfo> {
  fn header(links: bool) -> &'static [&'static str] {
    if links {
      &["Alerter", "Type", "Enabled", "Tags", "Link"]
    } else {
      &["Alerter", "Type", "Enabled", "Tags"]
    }
  }
  fn row(self, links: bool) -> Vec<comfy_table::Cell> {
    let mut row = vec![
      Cell::new(self.name).add_attribute(Attribute::Bold),
      Cell::new(self.info.endpoint_type),
      if self.info.enabled {
        Cell::new(self.info.enabled.to_string()).fg(Color::Green)
      } else {
        Cell::new(self.info.enabled.to_string()).fg(Color::Red)
      },
      Cell::new(self.tags.join(", ")),
    ];
    if links {
      row.push(Cell::new(resource_link(
        &cli_config().host,
        ResourceTargetVariant::Alerter,
        &self.id,
      )));
    }
    row
  }
}

impl PrintTable for Terminal {
  fn header(_links: bool) -> &'static [&'static str] {
    &["Terminal", "Target", "Command", "Size", "Created"]
  }
  fn row(self, _links: bool) -> Vec<comfy_table::Cell> {
    vec![
      Cell::new(self.name).add_attribute(Attribute::Bold),
      Cell::new(format!("{:?}", self.target)),
      Cell::new(self.command),
      Cell::new(if self.stored_size_kb < 1.0 {
        format!("{:.1} KiB", self.stored_size_kb)
      } else {
        format!("{:.} KiB", self.stored_size_kb)
      }),
      Cell::new(
        format_timetamp(self.created_at)
          .unwrap_or_else(|_| String::from("Invalid created at")),
      ),
    ]
  }
}

impl PrintTable for Schedule {
  fn header(links: bool) -> &'static [&'static str] {
    if links {
      &["Name", "Type", "Next Run", "Tags", "Link"]
    } else {
      &["Name", "Type", "Next Run", "Tags"]
    }
  }
  fn row(self, links: bool) -> Vec<comfy_table::Cell> {
    let next_run = if let Some(ts) = self.next_scheduled_run {
      Cell::new(
        format_timetamp(ts)
          .unwrap_or_else(|_| String::from("Invalid next ts")),
      )
      .add_attribute(Attribute::Bold)
    } else {
      Cell::new(String::from("None"))
    };
    let (resource_type, id) = self.target.extract_variant_id();
    let mut res = vec![
      Cell::new(self.name).add_attribute(Attribute::Bold),
      Cell::new(self.target.extract_variant_id().0),
      next_run,
      Cell::new(self.tags.join(", ")),
    ];
    if links {
      res.push(Cell::new(resource_link(
        &cli_config().host,
        resource_type,
        id,
      )));
    }
    res
  }
}
