use std::{collections::HashMap, future::Future};

use anyhow::{Context, anyhow};
use database::{
  bson::Document, mongo_indexed::doc, mungos::find::find_collect,
};
use futures_util::{TryStreamExt, future::BoxFuture};
use indexmap::IndexSet;
use komodo_client::{
  api::read::GetPermission,
  entities::{
    ResourceTarget,
    action::Action,
    alerter::Alerter,
    build::Build,
    builder::Builder,
    deployment::Deployment,
    permission::SpecificPermission,
    permission::{
      Permission, PermissionLevel, PermissionLevelAndSpecifics,
    },
    procedure::Procedure,
    repo::Repo,
    resource::Resource,
    server::Server,
    stack::Stack,
    swarm::Swarm,
    sync::ResourceSync,
    user::User,
  },
};
use mogh_resolver::Resolve;

use crate::{
  api::read::ReadArgs,
  config::core_config,
  helpers::query::{get_user_user_groups, user_target_query},
  resource::{KomodoResource, get, list_all_resources},
  state::db_client,
};

pub async fn get_check_permissions<T: KomodoResource>(
  id_or_name: &str,
  user: &User,
  required_permissions: PermissionLevelAndSpecifics,
) -> anyhow::Result<Resource<T::Config, T::Info>> {
  let resource = get::<T>(id_or_name).await?;

  // Allow all if admin
  if user.admin {
    return Ok(resource);
  }

  let user_permissions =
    get_user_permission_on_resource::<T>(user, &resource.id).await?;

  if (
    // Allow if its just read or below, and transparent mode enabled
    (required_permissions.level <= PermissionLevel::Read && core_config().transparent_mode)
    // Allow if resource has base permission level greater than or equal to required permission level
    || resource.base_permission.level >= required_permissions.level
  ) && user_permissions
    .fulfills_specific(&required_permissions.specific)
  {
    return Ok(resource);
  }

  if user_permissions.fulfills(&required_permissions) {
    Ok(resource)
  } else {
    Err(anyhow!(
      "User does not have required permissions on this {}. Must have at least {} permissions{}",
      T::resource_type(),
      required_permissions.level,
      if required_permissions.specific.is_empty() {
        String::new()
      } else {
        format!(
          ", as well as these specific permissions: [{}]",
          required_permissions.specifics_for_log()
        )
      }
    ))
  }
}

pub fn get_user_permission_on_resource<'a, T: KomodoResource>(
  user: &'a User,
  resource_id: &'a str,
) -> BoxFuture<'a, anyhow::Result<PermissionLevelAndSpecifics>> {
  Box::pin(async move {
    // Admin returns early with max permissions
    if user.admin {
      return Ok(PermissionLevel::Write.all());
    }

    let resource_type = T::resource_type();
    let resource = get::<T>(resource_id).await?;
    let initial_specific = if let Some(additional_target) =
      T::inherit_specific_permissions_from(&resource)
      // Ensure target is actually assigned
      && !additional_target.is_empty()
    {
      GetPermission {
        target: additional_target,
      }
      .resolve(&ReadArgs { user: user.clone() })
      .await
      .map_err(|e| e.error)
      .context("failed to get user permission on additional target")?
      .specific
    } else {
      IndexSet::new()
    };

    let mut permission = PermissionLevelAndSpecifics {
      level: if core_config().transparent_mode {
        PermissionLevel::Read
      } else {
        PermissionLevel::None
      },
      specific: initial_specific,
    };

    // Add in the resource level global base permissions
    if resource.base_permission.level > permission.level {
      permission.level = resource.base_permission.level;
    }
    permission
      .specific
      .extend(resource.base_permission.specific);

    // Overlay users base on resource variant
    if let Some(user_permission) =
      user.all.get(&resource_type).cloned()
    {
      if user_permission.level > permission.level {
        permission.level = user_permission.level;
      }
      permission.specific.extend(user_permission.specific);
    }

    // Overlay any user groups base on resource variant
    let groups = get_user_user_groups(&user.id).await?;
    for group in &groups {
      if let Some(group_permission) =
        group.all.get(&resource_type).cloned()
      {
        if group_permission.level > permission.level {
          permission.level = group_permission.level;
        }
        permission.specific.extend(group_permission.specific);
      }
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
    // get the max resource permission user has between personal / any user groups
    .fold(permission, |mut permission, resource_permission| {
      if resource_permission.level > permission.level {
        permission.level = resource_permission.level
      }
      permission.specific.extend(resource_permission.specific);
      permission
    });
    Ok(permission)
  })
}

/// Precomputed user permissions for listing resources of a type.
/// Load with [load_list_permits], then check visibility of
/// each resource with [ListPermits::permitted].
pub enum ListPermits {
  /// The user can see all resources of the type,
  /// eg. admin, transparent mode, or 'all' access on the variant.
  Unrestricted,
  /// Visibility must be checked per-resource
  /// against the permissions table.
  Fine(Box<FineListPermits>),
}

pub struct FineListPermits {
  required: PermissionLevelAndSpecifics,
  base: PermissionLevelAndSpecifics,
  permission_by_resource_id: HashMap<String, Permission>,
  additional_specific_cache:
    HashMap<ResourceTarget, IndexSet<SpecificPermission>>,
  user: User,
}

pub async fn load_list_permits<T: KomodoResource>(
  user: &User,
  required: PermissionLevelAndSpecifics,
) -> anyhow::Result<ListPermits> {
  // Check admin
  if user.admin {
    return Ok(ListPermits::Unrestricted);
  }

  let mut base = PermissionLevelAndSpecifics {
    level: if core_config().transparent_mode {
      PermissionLevel::Read
    } else {
      PermissionLevel::None
    },
    specific: Default::default(),
  };

  // 'transparent_mode' early return.
  if base.fulfills(&required) {
    return Ok(ListPermits::Unrestricted);
  }

  let resource_type = T::resource_type();

  // Check user 'all' on variant
  if let Some(all_permission) = user.all.get(&resource_type) {
    base.elevate(all_permission);
    // 'user.all' early return.
    if base.fulfills(&required) {
      return Ok(ListPermits::Unrestricted);
    }
  }

  // Check user groups 'all' on variant
  let groups = get_user_user_groups(&user.id).await?;
  for group in &groups {
    if let Some(all_permission) = group.all.get(&resource_type) {
      base.elevate(all_permission);
      // 'group.all' early return.
      if base.fulfills(&required) {
        return Ok(ListPermits::Unrestricted);
      }
    }
  }

  // Pull any permissions on the variant using the permissions table
  let permissions = find_collect(
    &db_client().permissions,
    doc! {
      "$or": user_target_query(&user.id, &groups)?,
      "resource_target.type": resource_type.as_ref(),
    },
    None,
  )
  .await
  .context("failed to query permissions on db")?;

  let permission_by_resource_id = permissions
    .into_iter()
    .map(|perm| {
      (
        perm.resource_target.extract_variant_id().1.to_string(),
        perm,
      )
    })
    .collect::<HashMap<_, _>>();

  Ok(ListPermits::Fine(
    FineListPermits {
      required,
      base,
      permission_by_resource_id,
      additional_specific_cache: Default::default(),
      user: user.clone(),
    }
    .into(),
  ))
}

impl ListPermits {
  /// Check whether the user can see the given resource.
  pub async fn permitted<T: KomodoResource>(
    &mut self,
    resource: &Resource<T::Config, T::Info>,
  ) -> anyhow::Result<bool> {
    let ListPermits::Fine(fine) = self else {
      return Ok(true);
    };

    let mut perm = if let Some(perm) =
      fine.permission_by_resource_id.get(&resource.id)
    {
      fine.base.join(perm)
    } else {
      fine.base.clone()
    };
    // Add in the resource level base permissions,
    // matching [get_user_permission_on_resource].
    perm.elevate(&resource.base_permission);
    // Check if already fulfils
    if perm.fulfills(&fine.required) {
      return Ok(true);
    }

    // Also check if fulfills with inherited specific
    let additional_target = if let Some(additional_target) =
      T::inherit_specific_permissions_from(resource)
      && !additional_target.is_empty()
    {
      additional_target
    } else {
      return Ok(false);
    };
    let additional_specific = match fine
      .additional_specific_cache
      .get(&additional_target)
      .cloned()
    {
      Some(specific) => specific,
      None => {
        let specific = GetPermission {
          target: additional_target.clone(),
        }
        .resolve(&ReadArgs {
          user: fine.user.clone(),
        })
        .await
        .map_err(|e| e.error)
        .context(
          "failed to get user permission on additional target",
        )?
        .specific;
        fine
          .additional_specific_cache
          .insert(additional_target, specific.clone());
        specific
      }
    };
    perm.specific.extend(additional_specific);
    Ok(perm.fulfills(&fine.required))
  }
}

pub async fn list_resources_for_user<T: KomodoResource>(
  filters: impl Into<Option<Document>>,
  limit: impl Into<Option<i64>>,
  skip: impl Into<Option<u64>>,
  user: &User,
  permission: PermissionLevelAndSpecifics,
) -> anyhow::Result<Vec<Resource<T::Config, T::Info>>> {
  let mut permits = load_list_permits::<T>(user, permission).await?;

  if let ListPermits::Unrestricted = permits {
    return list_all_resources::<T>(filters, limit, skip).await;
  }

  list_resources_with_permits::<T, _>(
    &mut permits,
    filters.into(),
    limit.into(),
    skip.into(),
    |resource| async move { Some(resource) },
  )
  .await
}

/// Drive the cursor directly, checking each resource against the
/// user permissions and the additional `filter` (eg. by state
/// computed from the in memory caches), with limit / skip applied
/// in memory after the filters. Stops pulling from the cursor as
/// soon as the limit is reached, and avoids collecting resources
/// the user cannot see.
///
/// `filter` receives each resource by value, and returns
/// `Some(resource)` to keep it in the results, or `None` to drop it.
pub async fn list_resources_with_permits<T: KomodoResource, F>(
  permits: &mut ListPermits,
  filters: Option<Document>,
  limit: Option<i64>,
  skip: Option<u64>,
  filter: impl Fn(Resource<T::Config, T::Info>) -> F,
) -> anyhow::Result<Vec<Resource<T::Config, T::Info>>>
where
  F: Future<Output = Option<Resource<T::Config, T::Info>>> + Send,
{
  let mut cursor = T::coll()
    .find(filters.unwrap_or_default())
    .sort(doc! { "name": 1 })
    .await
    .with_context(|| {
      format!("Failed to query db for {}s", T::resource_type())
    })?;

  let limit = limit.unwrap_or_default().max(0) as usize;
  let skip = skip.unwrap_or_default();
  let mut skipped = 0;
  let mut resources = Vec::new();

  while let Some(resource) = cursor
    .try_next()
    .await
    .context("Failed to pull next resource from db cursor")?
  {
    if !permits.permitted::<T>(&resource).await? {
      continue;
    }
    let Some(resource) = filter(resource).await else {
      continue;
    };
    if skipped < skip {
      skipped += 1;
      continue;
    }
    resources.push(resource);
    if limit != 0 && resources.len() >= limit {
      break;
    }
  }

  Ok(resources)
}

/// Returns None if still no need to filter by resource id (eg transparent mode, group membership with all access).
pub async fn list_resource_ids_for_user<T: KomodoResource>(
  filters: Option<Document>,
  limit: impl Into<Option<i64>>,
  skip: impl Into<Option<u64>>,
  user: &User,
  permission: PermissionLevelAndSpecifics,
) -> anyhow::Result<Option<Vec<String>>> {
  let mut permits = load_list_permits::<T>(user, permission).await?;

  if let ListPermits::Unrestricted = permits {
    return Ok(None);
  }

  let ids = list_resources_with_permits::<T, _>(
    &mut permits,
    filters,
    limit.into(),
    skip.into(),
    |resource| async move { Some(resource) },
  )
  .await?
  .into_iter()
  .map(|resource| resource.id)
  .collect();

  Ok(Some(ids))
}

/// Usable for Update and Alert queries.
pub async fn user_resource_target_query(
  user: &User,
  incoming_query: Option<Document>,
) -> anyhow::Result<Option<Document>> {
  if user.admin || core_config().transparent_mode {
    Ok(incoming_query)
  } else {
    let swarm_query = list_resource_ids_for_user::<Swarm>(
      None,
      None,
      None,
      user,
      PermissionLevel::Read.into(),
    )
    .await?
    .map(|ids| {
      doc! {
        "target.type": "Swarm", "target.id": { "$in": ids }
      }
    })
    // If 'list_resource_ids_for_user' returns Ok(None), user
    // can read all resources of this type.
    .unwrap_or_else(|| doc! { "target.type": "Swarm" });

    let server_query = list_resource_ids_for_user::<Server>(
      None,
      None,
      None,
      user,
      PermissionLevel::Read.into(),
    )
    .await?
    .map(|ids| {
      doc! {
        "target.type": "Server", "target.id": { "$in": ids }
      }
    })
    .unwrap_or_else(|| doc! { "target.type": "Server" });

    let stack_query = list_resource_ids_for_user::<Stack>(
      None,
      None,
      None,
      user,
      PermissionLevel::Read.into(),
    )
    .await?
    .map(|ids| {
      doc! {
        "target.type": "Stack", "target.id": { "$in": ids }
      }
    })
    .unwrap_or_else(|| doc! { "target.type": "Stack" });

    let deployment_query = list_resource_ids_for_user::<Deployment>(
      None,
      None,
      None,
      user,
      PermissionLevel::Read.into(),
    )
    .await?
    .map(|ids| {
      doc! {
        "target.type": "Deployment", "target.id": { "$in": ids }
      }
    })
    .unwrap_or_else(|| doc! { "target.type": "Deployment" });

    let build_query = list_resource_ids_for_user::<Build>(
      None,
      None,
      None,
      user,
      PermissionLevel::Read.into(),
    )
    .await?
    .map(|ids| {
      doc! {
        "target.type": "Build", "target.id": { "$in": ids }
      }
    })
    .unwrap_or_else(|| doc! { "target.type": "Build" });

    let repo_query = list_resource_ids_for_user::<Repo>(
      None,
      None,
      None,
      user,
      PermissionLevel::Read.into(),
    )
    .await?
    .map(|ids| {
      doc! {
        "target.type": "Repo", "target.id": { "$in": ids }
      }
    })
    .unwrap_or_else(|| doc! { "target.type": "Repo" });

    let procedure_query = list_resource_ids_for_user::<Procedure>(
      None,
      None,
      None,
      user,
      PermissionLevel::Read.into(),
    )
    .await?
    .map(|ids| {
      doc! {
        "target.type": "Procedure", "target.id": { "$in": ids }
      }
    })
    .unwrap_or_else(|| doc! { "target.type": "Procedure" });

    let action_query = list_resource_ids_for_user::<Action>(
      None,
      None,
      None,
      user,
      PermissionLevel::Read.into(),
    )
    .await?
    .map(|ids| {
      doc! {
        "target.type": "Action", "target.id": { "$in": ids }
      }
    })
    .unwrap_or_else(|| doc! { "target.type": "Action" });

    let builder_query = list_resource_ids_for_user::<Builder>(
      None,
      None,
      None,
      user,
      PermissionLevel::Read.into(),
    )
    .await?
    .map(|ids| {
      doc! {
        "target.type": "Builder", "target.id": { "$in": ids }
      }
    })
    .unwrap_or_else(|| doc! { "target.type": "Builder" });

    let alerter_query = list_resource_ids_for_user::<Alerter>(
      None,
      None,
      None,
      user,
      PermissionLevel::Read.into(),
    )
    .await?
    .map(|ids| {
      doc! {
        "target.type": "Alerter", "target.id": { "$in": ids }
      }
    })
    .unwrap_or_else(|| doc! { "target.type": "Alerter" });

    let resource_sync_query =
      list_resource_ids_for_user::<ResourceSync>(
        None,
        None,
        None,
        user,
        PermissionLevel::Read.into(),
      )
      .await?
      .map(|ids| {
        doc! {
          "target.type": "ResourceSync", "target.id": { "$in": ids }
        }
      })
      // If 'list_resource_ids_for_user' returns Ok(None), user
      // can read all resources of this type.
      .unwrap_or_else(|| doc! { "target.type": "ResourceSync" });

    let query = if let Some(query) = incoming_query {
      doc! {
        "$and": [
          {
            "$or": [
              swarm_query,
              server_query,
              stack_query,
              deployment_query,
              build_query,
              repo_query,
              procedure_query,
              action_query,
              builder_query,
              alerter_query,
              resource_sync_query,
            ]
          },
          query
        ]
      }
    } else {
      doc! {
        "$or": [
          swarm_query,
          server_query,
          stack_query,
          deployment_query,
          build_query,
          repo_query,
          procedure_query,
          action_query,
          builder_query,
          alerter_query,
          resource_sync_query,
        ]
      }
    };

    Ok(Some(query))
  }
}

pub async fn check_user_target_access(
  target: &ResourceTarget,
  user: &User,
  required_permissions: PermissionLevelAndSpecifics,
) -> anyhow::Result<()> {
  match target {
    ResourceTarget::System(_) => {
      return Err(anyhow!(
        "user must be admin to view system updates"
      ));
    }
    ResourceTarget::Swarm(id) => {
      get_check_permissions::<Swarm>(id, user, required_permissions)
        .await?;
    }
    ResourceTarget::Server(id) => {
      get_check_permissions::<Server>(id, user, required_permissions)
        .await?;
    }
    ResourceTarget::Stack(id) => {
      get_check_permissions::<Stack>(id, user, required_permissions)
        .await?;
    }
    ResourceTarget::Deployment(id) => {
      get_check_permissions::<Deployment>(
        id,
        user,
        required_permissions,
      )
      .await?;
    }
    ResourceTarget::Build(id) => {
      get_check_permissions::<Build>(id, user, required_permissions)
        .await?;
    }
    ResourceTarget::Repo(id) => {
      get_check_permissions::<Repo>(id, user, required_permissions)
        .await?;
    }
    ResourceTarget::Procedure(id) => {
      get_check_permissions::<Procedure>(
        id,
        user,
        required_permissions,
      )
      .await?;
    }
    ResourceTarget::Action(id) => {
      get_check_permissions::<Action>(id, user, required_permissions)
        .await?;
    }
    ResourceTarget::ResourceSync(id) => {
      get_check_permissions::<ResourceSync>(
        id,
        user,
        required_permissions,
      )
      .await?;
    }
    ResourceTarget::Builder(id) => {
      get_check_permissions::<Builder>(
        id,
        user,
        required_permissions,
      )
      .await?;
    }
    ResourceTarget::Alerter(id) => {
      get_check_permissions::<Alerter>(
        id,
        user,
        required_permissions,
      )
      .await?;
    }
  }
  Ok(())
}
