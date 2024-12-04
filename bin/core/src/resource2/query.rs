use std::{
  collections::{HashMap, HashSet},
  str::FromStr,
};

use anyhow::{anyhow, Context};
use futures::future::{join_all, FutureExt};
use komodo_client::{
  entities::{
    action::*,
    build::*,
    builder::*,
    deployment::*,
    permission::PermissionLevel,
    procedure::*,
    repo::*,
    resource::{
      AddFilters, Resource, ResourceQuery as ResourceQuerySchema,
    },
    server::*,
    server_template::*,
    stack::*,
    sync::*,
    tag::Tag,
    user::User,
    ResourceTargetVariant,
  },
  parsers::parse_string_list,
};
use mongo_indexed::Document;
use mungos::{
  find::find_collect,
  mongodb::{
    bson::{doc, oid::ObjectId},
    options::FindOptions,
    Collection,
  },
};
use serde::{de::DeserializeOwned, Serialize};

use crate::{
  config::core_config,
  helpers::query::{
    get_user_user_groups, id_or_name_filter, user_target_query,
  },
  state::{
    action_state_cache, action_states, build_state_cache, db_client,
    deployment_status_cache, procedure_state_cache, repo_state_cache,
    repo_status_cache, resource_sync_state_cache,
    server_status_cache, stack_status_cache,
  },
};

use super::ResourceBase;

pub trait ResourceQuery: ResourceBase {
  type Config: Default
    + Send
    + Sync
    + Serialize
    + DeserializeOwned
    + 'static;
  type Info: Default
    + Send
    + Sync
    + Serialize
    + DeserializeOwned
    + 'static;
  type ListItem: Send;
  type QuerySpecifics: AddFilters + Default + std::fmt::Debug;

  fn coll() -> &'static Collection<Resource<Self::Config, Self::Info>>;

  async fn to_list_item(
    resource: Resource<Self::Config, Self::Info>,
  ) -> Self::ListItem;
}

#[instrument(level = "debug")]
pub async fn get<R: ResourceQuery>(
  id_or_name: &str,
) -> anyhow::Result<Resource<R::Config, R::Info>> {
  R::coll()
    .find_one(id_or_name_filter(id_or_name))
    .await
    .context("failed to query db for resource")?
    .with_context(|| {
      format!(
        "did not find any {} matching {id_or_name}",
        R::resource_type()
      )
    })
}

#[instrument(level = "debug")]
pub async fn get_check_permissions<R: ResourceQuery>(
  id_or_name: &str,
  user: &User,
  permission_level: PermissionLevel,
) -> anyhow::Result<Resource<R::Config, R::Info>> {
  let resource = get::<R>(id_or_name).await?;
  if user.admin
    // Allow if its just read or below, and transparent mode enabled
    || (permission_level <= PermissionLevel::Read
      && core_config().transparent_mode)
    // Allow if resource has base permission level greater than or equal to required permission level
    || resource.base_permission >= permission_level
  {
    return Ok(resource);
  }
  let permissions =
    get_user_permission_on_resource::<R>(user, &resource.id).await?;
  if permissions >= permission_level {
    Ok(resource)
  } else {
    Err(anyhow!(
      "User does not have required permissions on this {}. Must have at least {permission_level} permissions",
      R::resource_type()
    ))
  }
}

#[instrument(level = "debug")]
pub async fn get_user_permission_on_resource<R: ResourceQuery>(
  user: &User,
  resource_id: &str,
) -> anyhow::Result<PermissionLevel> {
  if user.admin {
    return Ok(PermissionLevel::Write);
  }

  let resource_type = R::resource_type();

  // Start with base of Read or None
  let mut base = if core_config().transparent_mode {
    PermissionLevel::Read
  } else {
    PermissionLevel::None
  };

  // Add in the resource level global base permission
  let resource_base = get::<R>(resource_id).await?.base_permission;
  if resource_base > base {
    base = resource_base;
  }

  // Overlay users base on resource variant
  if let Some(level) = user.all.get(&resource_type).cloned() {
    if level > base {
      base = level;
    }
  }
  if base == PermissionLevel::Write {
    // No reason to keep going if already Write at this point.
    return Ok(PermissionLevel::Write);
  }

  // Overlay any user groups base on resource variant
  let groups = get_user_user_groups(&user.id).await?;
  for group in &groups {
    if let Some(level) = group.all.get(&resource_type).cloned() {
      if level > base {
        base = level;
      }
    }
  }
  if base == PermissionLevel::Write {
    // No reason to keep going if already Write at this point.
    return Ok(PermissionLevel::Write);
  }

  // Overlay any specific permissions
  let permission = find_collect(
    &db_client().permissions,
    doc! {


      "$or": user_target_query(&user.id, &groups)?,
      "resource_target.type": resource_type.as_ref(),
      "resource_target.id": resource_id
    },
    None,
  )
  .await
  .context("failed to query db for permissions")?
  .into_iter()
  // get the max permission user has between personal / any user groups
  .fold(base, |level, permission| {
    if permission.level > level {
      permission.level
    } else {
      level
    }
  });
  Ok(permission)
}

// ======
// LIST
// ======

/// Returns None if still no need to filter by resource id (eg transparent mode, group membership with all access).
#[instrument(level = "debug")]
pub async fn get_resource_object_ids_for_user<R: ResourceQuery>(
  user: &User,
) -> anyhow::Result<Option<Vec<ObjectId>>> {
  get_resource_ids_for_user::<R>(user).await.map(|ids| {
    ids.map(|ids| {
      ids
        .into_iter()
        .flat_map(|id| ObjectId::from_str(&id))
        .collect()
    })
  })
}

/// Returns None if still no need to filter by resource id (eg transparent mode, group membership with all access).
#[instrument(level = "debug")]
pub async fn get_resource_ids_for_user<R: ResourceQuery>(
  user: &User,
) -> anyhow::Result<Option<Vec<String>>> {
  // Check admin or transparent mode
  if user.admin || core_config().transparent_mode {
    return Ok(None);
  }

  let resource_type = R::resource_type();

  // Check user 'all' on variant
  if let Some(level) = user.all.get(&resource_type).cloned() {
    if level > PermissionLevel::None {
      return Ok(None);
    }
  }

  // Check user groups 'all' on variant
  let groups = get_user_user_groups(&user.id).await?;
  for group in &groups {
    if let Some(level) = group.all.get(&resource_type).cloned() {
      if level > PermissionLevel::None {
        return Ok(None);
      }
    }
  }

  let (base, perms) = tokio::try_join!(
    // Get any resources with non-none base permission,
    find_collect(
      R::coll(),
      doc! { "base_permission": { "$exists": true, "$ne": "None" } },
      None,
    )
    .map(|res| res.with_context(|| format!(
      "failed to query {resource_type} on db"
    ))),
    // And any ids using the permissions table
    find_collect(
      &db_client().permissions,
      doc! {


        "$or": user_target_query(&user.id, &groups)?,
        "resource_target.type": resource_type.as_ref(),
        "level": { "$exists": true, "$ne": "None" }
      },
      None,
    )
    .map(|res| res.context("failed to query permissions on db"))
  )?;

  // Add specific ids
  let ids = perms
    .into_iter()
    .map(|p| p.resource_target.extract_variant_id().1.to_string())
    // Chain in the ones with non-None base permissions
    .chain(base.into_iter().map(|res| res.id))
    // collect into hashset first to remove any duplicates
    .collect::<HashSet<_>>();

  Ok(Some(ids.into_iter().collect()))
}

#[instrument(level = "debug")]
pub async fn list_for_user<R: ResourceQuery>(
  mut query: ResourceQuerySchema<R::QuerySpecifics>,
  user: &User,
  all_tags: &[Tag],
) -> anyhow::Result<Vec<R::ListItem>> {
  validate_resource_query_tags(&mut query, all_tags)?;
  let mut filters = Document::new();
  query.add_filters(&mut filters);
  list_for_user_using_document::<R>(filters, user).await
}

#[instrument(level = "debug")]
pub async fn list_for_user_using_pattern<R: ResourceQuery>(
  pattern: &str,
  query: ResourceQuerySchema<R::QuerySpecifics>,
  user: &User,
  all_tags: &[Tag],
) -> anyhow::Result<Vec<R::ListItem>> {
  let list = list_full_for_user_using_pattern::<R>(
    pattern, query, user, all_tags,
  )
  .await?
  .into_iter()
  .map(|resource| R::to_list_item(resource));
  Ok(join_all(list).await)
}

#[instrument(level = "debug")]
pub async fn list_for_user_using_document<R: ResourceQuery>(
  filters: Document,
  user: &User,
) -> anyhow::Result<Vec<R::ListItem>> {
  let list = list_full_for_user_using_document::<R>(filters, user)
    .await?
    .into_iter()
    .map(|resource| R::to_list_item(resource));
  Ok(join_all(list).await)
}

/// Lists full resource matching wildcard syntax,
/// or regex if wrapped with "\\"
///
/// ## Example
/// ```
/// let items = list_full_for_user_using_match_string::<Build>("foo-*", Default::default(), user, all_tags).await?;
/// let items = list_full_for_user_using_match_string::<Build>("\\^foo-.*$\\", Default::default(), user, all_tags).await?;
/// ```
#[instrument(level = "debug")]
pub async fn list_full_for_user_using_pattern<R: ResourceQuery>(
  pattern: &str,
  query: ResourceQuerySchema<R::QuerySpecifics>,
  user: &User,
  all_tags: &[Tag],
) -> anyhow::Result<Vec<Resource<R::Config, R::Info>>> {
  let resources =
    list_full_for_user::<R>(query, user, all_tags).await?;

  let patterns = parse_string_list(pattern);
  let mut names = HashSet::<String>::new();

  for pattern in patterns {
    if pattern.starts_with('\\') && pattern.ends_with('\\') {
      let regex = regex::Regex::new(&pattern[1..(pattern.len() - 1)])
        .context("Regex matching string invalid")?;
      for resource in &resources {
        if regex.is_match(&resource.name) {
          names.insert(resource.name.clone());
        }
      }
    } else {
      let wildcard = wildcard::Wildcard::new(pattern.as_bytes())
        .context("Wildcard matching string invalid")?;
      for resource in &resources {
        if wildcard.is_match(resource.name.as_bytes()) {
          names.insert(resource.name.clone());
        }
      }
    };
  }

  Ok(
    resources
      .into_iter()
      .filter(|resource| names.contains(resource.name.as_str()))
      .collect(),
  )
}

#[instrument(level = "debug")]
pub async fn list_full_for_user<R: ResourceQuery>(
  mut query: ResourceQuerySchema<R::QuerySpecifics>,
  user: &User,
  all_tags: &[Tag],
) -> anyhow::Result<Vec<Resource<R::Config, R::Info>>> {
  validate_resource_query_tags(&mut query, all_tags)?;
  let mut filters = Document::new();
  query.add_filters(&mut filters);
  list_full_for_user_using_document::<R>(filters, user).await
}

#[instrument(level = "debug")]
pub async fn list_full_for_user_using_document<R: ResourceQuery>(
  mut filters: Document,
  user: &User,
) -> anyhow::Result<Vec<Resource<R::Config, R::Info>>> {
  if let Some(ids) =
    get_resource_object_ids_for_user::<R>(user).await?
  {
    filters.insert("_id", doc! { "$in": ids });
  }
  find_collect(
    R::coll(),
    filters,
    FindOptions::builder().sort(doc! { "name": 1 }).build(),
  )
  .await
  .with_context(|| {
    format!("failed to pull {}s from mongo", R::resource_type())
  })
}

pub type IdResourceMap<R> = HashMap<
  String,
  Resource<<R as ResourceQuery>::Config, <R as ResourceQuery>::Info>,
>;

#[instrument(level = "debug")]
pub async fn get_id_to_resource_map<R: ResourceQuery>(
  id_to_tags: &HashMap<String, Tag>,
  match_tags: &[String],
) -> anyhow::Result<IdResourceMap<R>> {
  let res = find_collect(R::coll(), None, None)
    .await
    .with_context(|| {
      format!("failed to pull {}s from mongo", R::resource_type())
    })?
    .into_iter()
    .filter(|resource| {
      if match_tags.is_empty() {
        return true;
      }
      for tag in match_tags.iter() {
        for resource_tag in &resource.tags {
          match ObjectId::from_str(resource_tag) {
            Ok(_) => match id_to_tags
              .get(resource_tag)
              .map(|tag| tag.name.as_str())
            {
              Some(name) => {
                if tag != name {
                  return false;
                }
              }
              None => return false,
            },
            Err(_) => {
              if resource_tag != tag {
                return false;
              }
            }
          }
        }
      }
      true
    })
    .map(|r| (r.id.clone(), r))
    .collect();
  Ok(res)
}

#[instrument(level = "debug")]
pub fn validate_resource_query_tags<S: Default + std::fmt::Debug>(
  query: &mut ResourceQuerySchema<S>,
  all_tags: &[Tag],
) -> anyhow::Result<()> {
  query.tags = query
    .tags
    .iter()
    .map(|tag| {
      all_tags
        .iter()
        .find(|t| t.name == *tag || t.id == *tag)
        .map(|tag| tag.id.clone())
        .with_context(|| {
          format!("No tag found matching name or id: {}", tag)
        })
    })
    .collect::<anyhow::Result<Vec<_>>>()?;
  Ok(())
}

// =================
//  IMPLEMENTATIONS
// =================

impl ResourceQuery for Server {
  type Config = ServerConfig;
  type Info = ();
  type ListItem = ServerListItem;
  type QuerySpecifics = ServerQuerySpecifics;

  fn coll() -> &'static Collection<Resource<Self::Config, Self::Info>>
  {
    &db_client().servers
  }

  async fn to_list_item(
    server: Resource<Self::Config, Self::Info>,
  ) -> Self::ListItem {
    let status = server_status_cache().get(&server.id).await;
    ServerListItem {
      name: server.name,
      id: server.id,
      tags: server.tags,
      resource_type: ResourceTargetVariant::Server,
      info: ServerListItemInfo {
        state: status.map(|s| s.state).unwrap_or_default(),
        region: server.config.region,
        address: server.config.address,
        send_unreachable_alerts: server
          .config
          .send_unreachable_alerts,
        send_cpu_alerts: server.config.send_cpu_alerts,
        send_mem_alerts: server.config.send_mem_alerts,
        send_disk_alerts: server.config.send_disk_alerts,
      },
    }
  }
}

impl ResourceQuery for Stack {
  type Config = StackConfig;
  type Info = StackInfo;
  type ListItem = StackListItem;
  type QuerySpecifics = StackQuerySpecifics;

  fn coll() -> &'static Collection<Resource<Self::Config, Self::Info>>
  {
    &db_client().stacks
  }

  async fn to_list_item(
    stack: Resource<Self::Config, Self::Info>,
  ) -> Self::ListItem {
    let status = stack_status_cache().get(&stack.id).await;
    let state =
      status.as_ref().map(|s| s.curr.state).unwrap_or_default();
    let project_name = stack.project_name(false);
    let services = status
      .as_ref()
      .map(|s| {
        s.curr
          .services
          .iter()
          .map(|service| StackServiceWithUpdate {
            service: service.service.clone(),
            image: service.image.clone(),
            update_available: service.update_available,
          })
          .collect::<Vec<_>>()
      })
      .unwrap_or_default();

    // This is only true if it is KNOWN to be true. so other cases are false.
    let (project_missing, status) =
      if stack.config.server_id.is_empty()
        || matches!(state, StackState::Down | StackState::Unknown)
      {
        (false, None)
      } else if let Some(status) = server_status_cache()
        .get(&stack.config.server_id)
        .await
        .as_ref()
      {
        if let Some(projects) = &status.projects {
          if let Some(project) = projects
            .iter()
            .find(|project| project.name == project_name)
          {
            (false, project.status.clone())
          } else {
            // The project doesn't exist
            (true, None)
          }
        } else {
          (false, None)
        }
      } else {
        (false, None)
      };

    StackListItem {
      id: stack.id,
      name: stack.name,
      tags: stack.tags,
      resource_type: ResourceTargetVariant::Stack,
      info: StackListItemInfo {
        state,
        status,
        services,
        project_missing,
        file_contents: !stack.config.file_contents.is_empty(),
        server_id: stack.config.server_id,
        missing_files: stack.info.missing_files,
        files_on_host: stack.config.files_on_host,
        git_provider: stack.config.git_provider,
        repo: stack.config.repo,
        branch: stack.config.branch,
        latest_hash: stack.info.latest_hash,
        deployed_hash: stack.info.deployed_hash,
      },
    }
  }
}

impl ResourceQuery for Deployment {
  type Config = DeploymentConfig;
  type Info = ();
  type ListItem = DeploymentListItem;
  type QuerySpecifics = DeploymentQuerySpecifics;

  fn coll() -> &'static Collection<Resource<Self::Config, Self::Info>>
  {
    &db_client().deployments
  }

  async fn to_list_item(
    deployment: Resource<Self::Config, Self::Info>,
  ) -> Self::ListItem {
    let status = deployment_status_cache().get(&deployment.id).await;
    let (build_image, build_id) = match deployment.config.image {
      DeploymentImage::Build { build_id, version } => {
        let (build_name, build_id, build_version) =
          super::query::get::<Build>(&build_id)
            .await
            .map(|b| (b.name, b.id, b.config.version))
            .unwrap_or((
              String::from("unknown"),
              String::new(),
              Default::default(),
            ));
        let version = if version.is_none() {
          build_version.to_string()
        } else {
          version.to_string()
        };
        (format!("{build_name}:{version}"), Some(build_id))
      }
      DeploymentImage::Image { image } => (image, None),
    };
    let (image, update_available) = status
      .as_ref()
      .and_then(|s| {
        s.curr.container.as_ref().map(|c| {
          (
            c.image
              .clone()
              .unwrap_or_else(|| String::from("Unknown")),
            s.curr.update_available,
          )
        })
      })
      .unwrap_or((build_image, false));
    DeploymentListItem {
      name: deployment.name,
      id: deployment.id,
      tags: deployment.tags,
      resource_type: ResourceTargetVariant::Deployment,
      info: DeploymentListItemInfo {
        state: status
          .as_ref()
          .map(|s| s.curr.state)
          .unwrap_or_default(),
        status: status.as_ref().and_then(|s| {
          s.curr.container.as_ref().and_then(|c| c.status.to_owned())
        }),
        image,
        update_available,
        server_id: deployment.config.server_id,
        build_id,
      },
    }
  }
}

impl ResourceQuery for Build {
  type Config = BuildConfig;
  type Info = BuildInfo;
  type ListItem = BuildListItem;
  type QuerySpecifics = BuildQuerySpecifics;

  fn coll() -> &'static Collection<Resource<Self::Config, Self::Info>>
  {
    &db_client().builds
  }

  async fn to_list_item(
    build: Resource<Self::Config, Self::Info>,
  ) -> Self::ListItem {
    let state = if action_states()
      .build
      .get(&build.id)
      .await
      .map(|s| s.get().map(|s| s.building))
      .transpose()
      .ok()
      .flatten()
      .unwrap_or_default()
    {
      BuildState::Building
    } else {
      build_state_cache().get(&build.id).await.unwrap_or_default()
    };
    BuildListItem {
      name: build.name,
      id: build.id,
      tags: build.tags,
      resource_type: ResourceTargetVariant::Build,
      info: BuildListItemInfo {
        last_built_at: build.info.last_built_at,
        version: build.config.version,
        builder_id: build.config.builder_id,
        git_provider: build.config.git_provider,
        image_registry_domain: build.config.image_registry.domain,
        repo: build.config.repo,
        branch: build.config.branch,
        built_hash: build.info.built_hash,
        latest_hash: build.info.latest_hash,
        state,
      },
    }
  }
}

impl ResourceQuery for Repo {
  type Config = RepoConfig;
  type Info = RepoInfo;
  type ListItem = RepoListItem;
  type QuerySpecifics = RepoQuerySpecifics;

  fn coll() -> &'static Collection<Resource<Self::Config, Self::Info>>
  {
    &db_client().repos
  }

  async fn to_list_item(
    repo: Resource<Self::Config, Self::Info>,
  ) -> Self::ListItem {
    let state = get_repo_state(&repo.id).await;
    let status =
      repo_status_cache().get(&repo.id).await.unwrap_or_default();
    RepoListItem {
      name: repo.name,
      id: repo.id,
      tags: repo.tags,
      resource_type: ResourceTargetVariant::Repo,
      info: RepoListItemInfo {
        server_id: repo.config.server_id,
        builder_id: repo.config.builder_id,
        last_pulled_at: repo.info.last_pulled_at,
        last_built_at: repo.info.last_built_at,
        git_provider: repo.config.git_provider,
        repo: repo.config.repo,
        branch: repo.config.branch,
        state,
        cloned_hash: status.latest_hash.clone(),
        cloned_message: status.latest_message.clone(),
        latest_hash: repo.info.latest_hash,
        built_hash: repo.info.built_hash,
      },
    }
  }
}

async fn get_repo_state(id: &String) -> RepoState {
  if let Some(state) = action_states()
    .repo
    .get(id)
    .await
    .and_then(|s| {
      s.get()
        .map(|s| {
          if s.cloning {
            Some(RepoState::Cloning)
          } else if s.pulling {
            Some(RepoState::Pulling)
          } else if s.building {
            Some(RepoState::Building)
          } else {
            None
          }
        })
        .ok()
    })
    .flatten()
  {
    return state;
  }
  repo_state_cache().get(id).await.unwrap_or_default()
}

impl ResourceQuery for Procedure {
  type Config = ProcedureConfig;
  type Info = ();
  type ListItem = ProcedureListItem;
  type QuerySpecifics = ProcedureQuerySpecifics;

  fn coll() -> &'static Collection<Resource<Self::Config, Self::Info>>
  {
    &db_client().procedures
  }

  async fn to_list_item(
    procedure: Resource<Self::Config, Self::Info>,
  ) -> Self::ListItem {
    let state = if action_states()
      .procedure
      .get(&procedure.id)
      .await
      .map(|s| s.get().map(|s| s.running))
      .transpose()
      .ok()
      .flatten()
      .unwrap_or_default()
    {
      ProcedureState::Running
    } else {
      procedure_state_cache()
        .get(&procedure.id)
        .await
        .unwrap_or_default()
    };
    ProcedureListItem {
      name: procedure.name,
      id: procedure.id,
      tags: procedure.tags,
      resource_type: ResourceTargetVariant::Procedure,
      info: ProcedureListItemInfo {
        stages: procedure.config.stages.len() as i64,
        state,
      },
    }
  }
}

impl ResourceQuery for Action {
  type Config = ActionConfig;
  type Info = ActionInfo;
  type ListItem = ActionListItem;
  type QuerySpecifics = ActionQuerySpecifics;

  fn coll() -> &'static Collection<Resource<Self::Config, Self::Info>>
  {
    &db_client().actions
  }

  async fn to_list_item(
    action: Resource<Self::Config, Self::Info>,
  ) -> Self::ListItem {
    let state = if action_states()
      .action
      .get(&action.id)
      .await
      .map(|s| s.get().map(|s| s.running))
      .transpose()
      .ok()
      .flatten()
      .unwrap_or_default()
    {
      ActionState::Running
    } else {
      action_state_cache()
        .get(&action.id)
        .await
        .unwrap_or_default()
    };
    ActionListItem {
      name: action.name,
      id: action.id,
      tags: action.tags,
      resource_type: ResourceTargetVariant::Action,
      info: ActionListItemInfo {
        state,
        last_run_at: action.info.last_run_at,
      },
    }
  }
}

impl ResourceQuery for Builder {
  type Config = BuilderConfig;
  type Info = ();
  type ListItem = BuilderListItem;
  type QuerySpecifics = BuilderQuerySpecifics;

  fn coll() -> &'static Collection<Resource<Self::Config, Self::Info>>
  {
    &db_client().builders
  }

  async fn to_list_item(
    builder: Resource<Self::Config, Self::Info>,
  ) -> Self::ListItem {
    let (builder_type, instance_type) = match builder.config {
      BuilderConfig::Url(_) => {
        (BuilderConfigVariant::Url.to_string(), None)
      }
      BuilderConfig::Server(config) => (
        BuilderConfigVariant::Server.to_string(),
        Some(config.server_id),
      ),
      BuilderConfig::Aws(config) => (
        BuilderConfigVariant::Aws.to_string(),
        Some(config.instance_type),
      ),
    };
    BuilderListItem {
      name: builder.name,
      id: builder.id,
      tags: builder.tags,
      resource_type: ResourceTargetVariant::Builder,
      info: BuilderListItemInfo {
        builder_type,
        instance_type,
      },
    }
  }
}

impl ResourceQuery for ServerTemplate {
  type Config = ServerTemplateConfig;
  type Info = ();
  type ListItem = ServerTemplateListItem;
  type QuerySpecifics = ServerTemplateQuerySpecifics;

  fn coll() -> &'static Collection<Resource<Self::Config, Self::Info>>
  {
    &db_client().server_templates
  }

  async fn to_list_item(
    server_template: Resource<Self::Config, Self::Info>,
  ) -> Self::ListItem {
    let (template_type, instance_type) = match server_template.config
    {
      ServerTemplateConfig::Aws(config) => (
        ServerTemplateConfigVariant::Aws.to_string(),
        Some(config.instance_type),
      ),
      ServerTemplateConfig::Hetzner(config) => (
        ServerTemplateConfigVariant::Hetzner.to_string(),
        Some(config.server_type.as_ref().to_string()),
      ),
    };
    ServerTemplateListItem {
      name: server_template.name,
      id: server_template.id,
      tags: server_template.tags,
      resource_type: ResourceTargetVariant::ServerTemplate,
      info: ServerTemplateListItemInfo {
        provider: template_type.to_string(),
        instance_type,
      },
    }
  }
}

async fn get_resource_sync_state(
  id: &String,
  data: &ResourceSyncInfo,
) -> ResourceSyncState {
  if let Some(state) = action_states()
    .resource_sync
    .get(id)
    .await
    .and_then(|s| {
      s.get()
        .map(|s| {
          if s.syncing {
            Some(ResourceSyncState::Syncing)
          } else {
            None
          }
        })
        .ok()
    })
    .flatten()
  {
    return state;
  }
  if data.pending_error.is_some() {
    return ResourceSyncState::Failed;
  }
  if !data.resource_updates.is_empty()
    || !data.variable_updates.is_empty()
    || !data.user_group_updates.is_empty()
    || data.pending_deploy.to_deploy > 0
  {
    return ResourceSyncState::Pending;
  }
  resource_sync_state_cache()
    .get(id)
    .await
    .unwrap_or_default()
}

impl ResourceQuery for ResourceSync {
  type Config = ResourceSyncConfig;
  type Info = ResourceSyncInfo;
  type ListItem = ResourceSyncListItem;
  type QuerySpecifics = ResourceSyncQuerySpecifics;

  fn coll() -> &'static Collection<Resource<Self::Config, Self::Info>>
  {
    &db_client().resource_syncs
  }

  async fn to_list_item(
    resource_sync: Resource<Self::Config, Self::Info>,
  ) -> Self::ListItem {
    let state =
      get_resource_sync_state(&resource_sync.id, &resource_sync.info)
        .await;
    ResourceSyncListItem {
      id: resource_sync.id,
      name: resource_sync.name,
      tags: resource_sync.tags,
      resource_type: ResourceTargetVariant::ResourceSync,
      info: ResourceSyncListItemInfo {
        file_contents: !resource_sync.config.file_contents.is_empty(),
        files_on_host: resource_sync.config.files_on_host,
        managed: resource_sync.config.managed,
        git_provider: resource_sync.config.git_provider,
        repo: resource_sync.config.repo,
        branch: resource_sync.config.branch,
        last_sync_ts: resource_sync.info.last_sync_ts,
        last_sync_hash: resource_sync.info.last_sync_hash,
        last_sync_message: resource_sync.info.last_sync_message,
        resource_path: resource_sync.config.resource_path,
        state,
      },
    }
  }
}
