use std::collections::HashMap;

use anyhow::{Context, anyhow};
use database::mungos::find::find_collect;
use database::{bson::Document, mongo_indexed::doc};
use futures_util::{FutureExt, future::BoxFuture};
use indexmap::IndexSet;
use komodo_client::entities::ResourceTarget;
use komodo_client::entities::action::Action;
use komodo_client::entities::alerter::Alerter;
use komodo_client::entities::build::Build;
use komodo_client::entities::builder::Builder;
use komodo_client::entities::deployment::Deployment;
use komodo_client::entities::permission::SpecificPermission;
use komodo_client::entities::procedure::Procedure;
use komodo_client::entities::repo::Repo;
use komodo_client::entities::server::Server;
use komodo_client::entities::stack::Stack;
use komodo_client::entities::swarm::Swarm;
use komodo_client::entities::sync::ResourceSync;
use komodo_client::{
  api::read::GetPermission,
  entities::{
    permission::{PermissionLevel, PermissionLevelAndSpecifics},
    resource::Resource,
    user::User,
  },
};
use resolver_api::Resolve;

use crate::resource::list_all_resources;
use crate::{
  api::read::ReadArgs,
  config::core_config,
  helpers::query::{get_user_user_groups, user_target_query},
  resource::{KomodoResource, get},
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

pub async fn list_resources_for_user<T: KomodoResource>(
  filters: impl Into<Option<Document>>,
  user: &User,
  permission: PermissionLevelAndSpecifics,
) -> anyhow::Result<Vec<Resource<T::Config, T::Info>>> {
  // Check admin
  if user.admin {
    return list_all_resources::<T>(filters).await;
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
  if base.fulfills(&permission) {
    return list_all_resources::<T>(filters).await;
  }

  let resource_type = T::resource_type();

  // Check user 'all' on variant
  if let Some(all_permission) = user.all.get(&resource_type) {
    base.elevate(all_permission);
    // 'user.all' early return.
    if base.fulfills(&permission) {
      return list_all_resources::<T>(filters).await;
    }
  }

  // Check user groups 'all' on variant
  let groups = get_user_user_groups(&user.id).await?;
  for group in &groups {
    if let Some(all_permission) = group.all.get(&resource_type) {
      base.elevate(all_permission);
      // 'group.all' early return.
      if base.fulfills(&permission) {
        return list_all_resources::<T>(filters).await;
      }
    }
  }

  let (all, permissions) = tokio::try_join!(
    list_all_resources::<T>(filters),
    // And any ids using the permissions table
    find_collect(
      &db_client().permissions,
      doc! {
        "$or": user_target_query(&user.id, &groups)?,
        "resource_target.type": resource_type.as_ref(),
      },
      None,
    )
    .map(|res| res.context("failed to query permissions on db"))
  )?;

  let permission_by_resource_id = permissions
    .into_iter()
    .map(|perm| {
      (
        perm.resource_target.extract_variant_id().1.to_string(),
        perm,
      )
    })
    .collect::<HashMap<_, _>>();

  let mut resources = Vec::new();
  let mut additional_specific_cache =
    HashMap::<ResourceTarget, IndexSet<SpecificPermission>>::new();

  for resource in all {
    let mut perm = if let Some(perm) =
      permission_by_resource_id.get(&resource.id)
    {
      base.join(perm)
    } else {
      base.clone()
    };
    // Check if already fulfils
    if perm.fulfills(&permission) {
      resources.push(resource);
      continue;
    }

    // Also check if fulfills with inherited specific
    let additional_target = if let Some(additional_target) =
      T::inherit_specific_permissions_from(&resource)
      && !additional_target.is_empty()
    {
      additional_target
    } else {
      continue;
    };
    let additional_specific = match additional_specific_cache
      .get(&additional_target)
      .cloned()
    {
      Some(specific) => specific,
      None => {
        let specific = GetPermission {
          target: additional_target.clone(),
        }
        .resolve(&ReadArgs { user: user.clone() })
        .await
        .map_err(|e| e.error)
        .context(
          "failed to get user permission on additional target",
        )?
        .specific;
        additional_specific_cache
          .insert(additional_target, specific.clone());
        specific
      }
    };
    perm.specific.extend(additional_specific);
    if perm.fulfills(&permission) {
      resources.push(resource);
    }
  }

  Ok(resources)
}

/// Returns None if still no need to filter by resource id (eg transparent mode, group membership with all access).
pub async fn list_resource_ids_for_user<T: KomodoResource>(
  filters: Option<Document>,
  user: &User,
  permission: PermissionLevelAndSpecifics,
) -> anyhow::Result<Option<Vec<String>>> {
  // Check admin
  if user.admin {
    return Ok(None);
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
  if base.fulfills(&permission) {
    return Ok(None);
  }

  let resource_type = T::resource_type();

  if let Some(all) = user.all.get(&resource_type) {
    base.elevate(all);
    // 'user.all' early return.
    if base.fulfills(&permission) {
      return Ok(None);
    }
  }

  // Check user groups 'all' on variant
  let groups = get_user_user_groups(&user.id).await?;
  for group in &groups {
    if let Some(all) = group.all.get(&resource_type) {
      base.elevate(all);
      // 'group.all' early return.
      if base.fulfills(&permission) {
        return Ok(None);
      }
    }
  }

  let ids = list_resources_for_user::<T>(filters, user, permission)
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
