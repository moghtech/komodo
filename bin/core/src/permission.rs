use std::collections::HashSet;

use anyhow::{Context, anyhow};
use futures::FutureExt;
use indexmap::IndexSet;
use komodo_client::entities::{
  permission::{PermissionLevel, PermissionLevelAndSpecifics},
  resource::Resource,
  user::User,
};
use mongo_indexed::doc;
use mungos::find::find_collect;

use crate::{
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

#[instrument(level = "debug")]
pub async fn get_user_permission_on_resource<T: KomodoResource>(
  user: &User,
  resource_id: &str,
) -> anyhow::Result<PermissionLevelAndSpecifics> {
  // Admin returns early with max permissions
  if user.admin {
    return Ok(PermissionLevel::Write.all());
  }

  let resource_type = T::resource_type();
  let resource = get::<T>(resource_id).await?;

  let mut permission = PermissionLevelAndSpecifics {
    level: if core_config().transparent_mode {
      PermissionLevel::Read
    } else {
      PermissionLevel::None
    },
    specific: IndexSet::new(),
  };

  // Add in the resource level global base permissions
  if resource.base_permission.level > permission.level {
    permission.level = resource.base_permission.level;
  }
  permission
    .specific
    .extend(resource.base_permission.specific);

  // Overlay users base on resource variant
  if let Some(user_permission) = user.all.get(&resource_type).cloned()
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
}

/// Returns None if still no need to filter by resource id (eg transparent mode, group membership with all access).
#[instrument(level = "debug")]
pub async fn get_resource_ids_for_user<T: KomodoResource>(
  user: &User,
) -> anyhow::Result<Option<Vec<String>>> {
  // Check admin or transparent mode
  if user.admin || core_config().transparent_mode {
    return Ok(None);
  }

  let resource_type = T::resource_type();

  // Check user 'all' on variant
  if let Some(permission) = user.all.get(&resource_type).cloned() {
    if permission.level > PermissionLevel::None {
      return Ok(None);
    }
  }

  // Check user groups 'all' on variant
  let groups = get_user_user_groups(&user.id).await?;
  for group in &groups {
    if let Some(permission) = group.all.get(&resource_type).cloned() {
      if permission.level > PermissionLevel::None {
        return Ok(None);
      }
    }
  }

  let (base, perms) = tokio::try_join!(
    // Get any resources with non-none base permission,
    find_collect(
      T::coll(),
      doc! { "$or": [
        { "base_permission": { "$in": ["Read", "Execute", "Write"] } },
        { "base_permission.level": { "$in": ["Read", "Execute", "Write"] } }
      ] },
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
        "level": { "$in": ["Read", "Execute", "Write"] }
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
