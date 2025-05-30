use std::{
  cmp::Ordering, collections::HashMap, fmt::Write, sync::OnceLock,
};

use anyhow::Context;
use formatting::{Color, bold, colored, muted};
use indexmap::{IndexMap, IndexSet};
use komodo_client::{
  api::{
    read::ListUserTargetPermissions,
    write::{
      CreateUserGroup, DeleteUserGroup, SetUsersInUserGroup,
      UpdatePermissionOnResourceType, UpdatePermissionOnTarget,
    },
  },
  entities::{
    ResourceTarget, ResourceTargetVariant,
    permission::{
      PermissionLevel, PermissionLevelAndSpecifics,
      SpecificPermission, UserTarget,
    },
    sync::DiffData,
    toml::{PermissionToml, UserGroupToml},
    update::Log,
    user::{User, sync_user},
    user_group::UserGroup,
  },
};
use mungos::find::find_collect;
use resolver_api::Resolve;
use serde::Serialize;

use crate::{
  api::{read::ReadArgs, write::WriteArgs},
  helpers::matcher::Matcher,
  state::db_client,
};

use super::{AllResourcesById, toml::TOML_PRETTY_OPTIONS};

/// Used to serialize user group
#[derive(Serialize)]
struct BasicUserGroupToml {
  name: String,
  #[serde(skip_serializing_if = "is_false")]
  everyone: bool,
  #[serde(skip_serializing_if = "Vec::is_empty")]
  users: Vec<String>,
}

fn is_false(b: &bool) -> bool {
  !b
}

/// Used to serialize user group
#[derive(Serialize)]
struct Permissions {
  permissions: Vec<PermissionToml>,
}

pub fn user_group_to_toml(
  user_group: UserGroupToml,
) -> anyhow::Result<String> {
  // Start with the basic body
  let basic = BasicUserGroupToml {
    name: user_group.name,
    everyone: user_group.everyone,
    users: if user_group.everyone {
      Vec::new()
    } else {
      user_group.users
    },
  };
  let basic = toml_pretty::to_string(&basic, TOML_PRETTY_OPTIONS)
    .context("failed to serialize user group to toml")?;
  let mut res = format!("[[user_group]]\n{basic}");

  // Add "all" permissions
  for (variant, PermissionLevelAndSpecifics { level, specific }) in
    user_group.all
  {
    write!(&mut res, "\nall.{variant} = ")
      .context("failed to serialize user group 'all' to toml")?;
    if specific.is_empty() {
      res.push('"');
      res.push_str(level.as_ref());
      res.push('"');
    } else {
      let specific = serde_json::to_string(&specific)
        .context(
          "failed to serialize user group specifics to... json?",
        )?
        .replace(",", ", ");
      write!(
        &mut res,
        "{{ level = \"{level}\", specific = {specific} }}"
      )
      .context(
        "failed to serialize user group 'all' with specifics to toml",
      )?;
    }
  }

  // End with resource permissions array
  res.push('\n');
  res.push_str(
    &toml_pretty::to_string(
      &Permissions {
        permissions: user_group.permissions,
      },
      TOML_PRETTY_OPTIONS,
    )
    .context("failed to serialize user group permissions to toml")?,
  );

  Ok(res)
}

pub struct UpdateItem {
  user_group: UserGroupToml,
  update_users: bool,
  all_diff:
    IndexMap<ResourceTargetVariant, PermissionLevelAndSpecifics>,
}

pub struct DeleteItem {
  id: String,
  name: String,
}

pub async fn get_updates_for_view(
  user_groups: Vec<UserGroupToml>,
  delete: bool,
  all_resources: &AllResourcesById,
) -> anyhow::Result<Vec<DiffData>> {
  let _curr = find_collect(&db_client().user_groups, None, None)
    .await
    .context("failed to query db for UserGroups")?;
  let mut curr = Vec::with_capacity(_curr.capacity());
  convert_user_groups(_curr.into_iter(), all_resources, &mut curr)
    .await?;
  let map = curr
    .into_iter()
    .map(|ug| (ug.1.name.clone(), ug))
    .collect::<HashMap<_, _>>();

  let mut diffs = Vec::<DiffData>::new();

  if delete {
    for (_id, user_group) in map.values() {
      if !user_groups.iter().any(|ug| ug.name == user_group.name) {
        diffs.push(DiffData::Delete {
          current: user_group_to_toml(user_group.clone())?,
        });
      }
    }
  }

  for mut user_group in user_groups {
    if user_group.everyone {
      user_group.users.clear();
    }

    user_group
      .permissions
      .retain(|p| p.level > PermissionLevel::None);

    user_group.permissions = expand_user_group_permissions(
      user_group.permissions,
      all_resources,
    )
    .await
    .with_context(|| {
      format!(
        "failed to expand user group {} permissions",
        user_group.name
      )
    })?;

    let (_original_id, original) =
      match map.get(&user_group.name).cloned() {
        Some(original) => original,
        None => {
          diffs.push(DiffData::Create {
            name: user_group.name.clone(),
            proposed: user_group_to_toml(user_group.clone())?,
          });
          continue;
        }
      };
    user_group.users.sort();

    let all_diff = diff_group_all(&original.all, &user_group.all);

    user_group.permissions.sort_by(sort_permissions);

    let update_users = user_group.users != original.users;
    let update_all = !all_diff.is_empty();
    let update_permissions =
      user_group.permissions != original.permissions;

    // only add log after diff detected
    if update_users || update_all || update_permissions {
      diffs.push(DiffData::Update {
        proposed: user_group_to_toml(user_group.clone())?,
        current: user_group_to_toml(original.clone())?,
      });
    }
  }

  Ok(diffs)
}

pub async fn get_updates_for_execution(
  user_groups: Vec<UserGroupToml>,
  delete: bool,
  all_resources: &AllResourcesById,
) -> anyhow::Result<(
  Vec<UserGroupToml>,
  Vec<UpdateItem>,
  Vec<DeleteItem>,
)> {
  let map = find_collect(&db_client().user_groups, None, None)
    .await
    .context("failed to query db for UserGroups")?
    .into_iter()
    .map(|ug| (ug.name.clone(), ug))
    .collect::<HashMap<_, _>>();

  let mut to_create = Vec::<UserGroupToml>::new();
  let mut to_update = Vec::<UpdateItem>::new();
  let mut to_delete = Vec::<DeleteItem>::new();

  if delete {
    for user_group in map.values() {
      if !user_groups.iter().any(|ug| ug.name == user_group.name) {
        to_delete.push(DeleteItem {
          id: user_group.id.clone(),
          name: user_group.name.clone(),
        });
      }
    }
  }

  if user_groups.is_empty() {
    return Ok((to_create, to_update, to_delete));
  }

  let id_to_user = find_collect(&db_client().users, None, None)
    .await
    .context("failed to query db for Users")?
    .into_iter()
    .map(|user| (user.id.clone(), user))
    .collect::<HashMap<_, _>>();

  for mut user_group in user_groups {
    if user_group.everyone {
      user_group.users.clear();
    }

    user_group
      .permissions
      .retain(|p| p.level > PermissionLevel::None);

    user_group.permissions = expand_user_group_permissions(
      user_group.permissions,
      all_resources,
    )
    .await
    .with_context(|| {
      format!(
        "Failed to expand user group {} permissions",
        user_group.name
      )
    })?;

    let original = match map.get(&user_group.name).cloned() {
      Some(original) => original,
      None => {
        to_create.push(user_group);
        continue;
      }
    };

    let mut original_users = original
      .users
      .into_iter()
      .filter_map(|user_id| {
        id_to_user.get(&user_id).map(|u| u.username.clone())
      })
      .collect::<Vec<_>>();

    let mut original_permissions = (ListUserTargetPermissions {
      user_target: UserTarget::UserGroup(original.id),
    })
    .resolve(&ReadArgs {
      user: sync_user().to_owned(),
    })
    .await
    .map_err(|e| e.error)
    .context("failed to query for existing UserGroup permissions")?
    .into_iter()
    .filter(|p| p.level > PermissionLevel::None)
    .map(|mut p| {
      // replace the ids with names
      match &mut p.resource_target {
        ResourceTarget::System(_) => {}
        ResourceTarget::Build(id) => {
          *id = all_resources
            .builds
            .get(id)
            .map(|b| b.name.clone())
            .unwrap_or_default()
        }
        ResourceTarget::Builder(id) => {
          *id = all_resources
            .builders
            .get(id)
            .map(|b| b.name.clone())
            .unwrap_or_default()
        }
        ResourceTarget::Deployment(id) => {
          *id = all_resources
            .deployments
            .get(id)
            .map(|b| b.name.clone())
            .unwrap_or_default()
        }
        ResourceTarget::Server(id) => {
          *id = all_resources
            .servers
            .get(id)
            .map(|b| b.name.clone())
            .unwrap_or_default()
        }
        ResourceTarget::Repo(id) => {
          *id = all_resources
            .repos
            .get(id)
            .map(|b| b.name.clone())
            .unwrap_or_default()
        }
        ResourceTarget::Alerter(id) => {
          *id = all_resources
            .alerters
            .get(id)
            .map(|b| b.name.clone())
            .unwrap_or_default()
        }
        ResourceTarget::Procedure(id) => {
          *id = all_resources
            .procedures
            .get(id)
            .map(|b| b.name.clone())
            .unwrap_or_default()
        }
        ResourceTarget::Action(id) => {
          *id = all_resources
            .actions
            .get(id)
            .map(|b| b.name.clone())
            .unwrap_or_default()
        }
        ResourceTarget::ResourceSync(id) => {
          *id = all_resources
            .syncs
            .get(id)
            .map(|b| b.name.clone())
            .unwrap_or_default()
        }
        ResourceTarget::Stack(id) => {
          *id = all_resources
            .stacks
            .get(id)
            .map(|b| b.name.clone())
            .unwrap_or_default()
        }
      }
      PermissionToml {
        target: p.resource_target,
        level: p.level,
        specific: p.specific,
      }
    })
    .collect::<Vec<_>>();

    original_users.sort();
    user_group.users.sort();

    let all_diff = diff_group_all(&original.all, &user_group.all);

    user_group.permissions.sort_by(sort_permissions);
    original_permissions.sort_by(sort_permissions);

    let update_users = user_group.users != original_users;

    // Extend permissions with any existing that have no target in incoming
    // This makes sure to set those permissions back to None.
    let to_remove = original_permissions
      .iter()
      .filter(|permission| {
        !user_group
          .permissions
          .iter()
          .any(|p| p.target == permission.target)
      })
      .map(|permission| PermissionToml {
        target: permission.target.clone(),
        level: PermissionLevel::None,
        specific: IndexSet::new(),
      })
      .collect::<Vec<_>>();
    user_group.permissions.extend(to_remove);

    // remove any permissions that already exist on original
    user_group.permissions.retain(|permission| {
      let Some(original_permission) = original_permissions
        .iter()
        .find(|p| p.target == permission.target)
      else {
        // not in original, keep it
        return true;
      };
      original_permission.level != permission.level
        || !specific_equal(
          &original_permission.specific,
          &permission.specific,
        )
    });

    // only push update after diff detected
    if update_users
      || !all_diff.is_empty()
      || !user_group.permissions.is_empty()
    {
      to_update.push(UpdateItem {
        user_group,
        update_users,
        all_diff: all_diff
          .into_iter()
          .map(|(k, (_, v))| (k, v))
          .collect(),
      });
    }
  }

  Ok((to_create, to_update, to_delete))
}

/// order permissions in deterministic way
fn sort_permissions(
  a: &PermissionToml,
  b: &PermissionToml,
) -> Ordering {
  let (a_t, a_id) = a.target.extract_variant_id();
  let (b_t, b_id) = b.target.extract_variant_id();
  match (a_t.cmp(&b_t), a_id.cmp(b_id)) {
    (Ordering::Greater, _) => Ordering::Greater,
    (Ordering::Less, _) => Ordering::Less,
    (_, Ordering::Greater) => Ordering::Greater,
    (_, Ordering::Less) => Ordering::Less,
    _ => Ordering::Equal,
  }
}

pub async fn run_updates(
  to_create: Vec<UserGroupToml>,
  to_update: Vec<UpdateItem>,
  to_delete: Vec<DeleteItem>,
) -> Option<Log> {
  if to_create.is_empty()
    && to_update.is_empty()
    && to_delete.is_empty()
  {
    return None;
  }

  let mut has_error = false;
  let mut log = String::from("running updates on UserGroups");

  // Create the non-existant user groups
  for user_group in to_create {
    // Create the user group
    if let Err(e) = (CreateUserGroup {
      name: user_group.name.clone(),
    })
    .resolve(&WriteArgs {
      user: sync_user().to_owned(),
    })
    .await
    {
      has_error = true;
      log.push_str(&format!(
        "\n{}: failed to create user group '{}' | {:#}",
        colored("ERROR", Color::Red),
        bold(&user_group.name),
        e.error
      ));
      continue;
    } else {
      log.push_str(&format!(
        "\n{}: {} user group '{}'",
        muted("INFO"),
        colored("created", Color::Green),
        bold(&user_group.name)
      ))
    };

    set_users(
      user_group.name.clone(),
      user_group.users,
      &mut log,
      &mut has_error,
    )
    .await;
    run_update_all(
      user_group.name.clone(),
      user_group.all,
      &mut log,
      &mut has_error,
    )
    .await;
    run_update_permissions(
      user_group.name,
      user_group.permissions,
      &mut log,
      &mut has_error,
    )
    .await;
  }

  // Update the existing user groups
  for UpdateItem {
    user_group,
    update_users,
    all_diff,
  } in to_update
  {
    if update_users {
      set_users(
        user_group.name.clone(),
        user_group.users,
        &mut log,
        &mut has_error,
      )
      .await;
    }
    if !all_diff.is_empty() {
      run_update_all(
        user_group.name.clone(),
        all_diff,
        &mut log,
        &mut has_error,
      )
      .await;
    }
    if !user_group.permissions.is_empty() {
      run_update_permissions(
        user_group.name,
        user_group.permissions,
        &mut log,
        &mut has_error,
      )
      .await;
    }
  }

  for user_group in to_delete {
    if let Err(e) = (DeleteUserGroup { id: user_group.id })
      .resolve(&WriteArgs {
        user: sync_user().to_owned(),
      })
      .await
    {
      has_error = true;
      log.push_str(&format!(
        "\n{}: failed to delete user group '{}' | {:#}",
        colored("ERROR", Color::Red),
        bold(&user_group.name),
        e.error
      ))
    } else {
      log.push_str(&format!(
        "\n{}: {} user group '{}'",
        muted("INFO"),
        colored("deleted", Color::Red),
        bold(&user_group.name)
      ))
    }
  }

  let stage = "Update UserGroups";
  Some(if has_error {
    Log::error(stage, log)
  } else {
    Log::simple(stage, log)
  })
}

async fn set_users(
  user_group: String,
  users: Vec<String>,
  log: &mut String,
  has_error: &mut bool,
) {
  if let Err(e) = (SetUsersInUserGroup {
    user_group: user_group.clone(),
    users,
  })
  .resolve(&WriteArgs {
    user: sync_user().to_owned(),
  })
  .await
  {
    *has_error = true;
    log.push_str(&format!(
      "\n{}: failed to set users in group {} | {:#}",
      colored("ERROR", Color::Red),
      bold(&user_group),
      e.error
    ))
  } else {
    log.push_str(&format!(
      "\n{}: {} user group '{}' users",
      muted("INFO"),
      colored("updated", Color::Blue),
      bold(&user_group)
    ))
  }
}

async fn run_update_all(
  user_group: String,
  all_diff: IndexMap<
    ResourceTargetVariant,
    PermissionLevelAndSpecifics,
  >,
  log: &mut String,
  has_error: &mut bool,
) {
  for (resource_type, permission) in all_diff {
    if let Err(e) = (UpdatePermissionOnResourceType {
      user_target: UserTarget::UserGroup(user_group.clone()),
      resource_type,
      permission,
    })
    .resolve(&WriteArgs {
      user: sync_user().to_owned(),
    })
    .await
    {
      *has_error = true;
      log.push_str(&format!(
        "\n{}: failed to set base permissions on {resource_type} in group {} | {:#}",
        colored("ERROR", Color::Red),
        bold(&user_group),
        e.error
      ))
    } else {
      log.push_str(&format!(
        "\n{}: {} user group '{}' base permissions on {resource_type}",
        muted("INFO"),
        colored("updated", Color::Blue),
        bold(&user_group)
      ))
    }
  }
}

async fn run_update_permissions(
  user_group: String,
  permissions: Vec<PermissionToml>,
  log: &mut String,
  has_error: &mut bool,
) {
  for PermissionToml {
    target,
    level,
    specific,
  } in permissions
  {
    if let Err(e) = (UpdatePermissionOnTarget {
      user_target: UserTarget::UserGroup(user_group.clone()),
      resource_target: target.clone(),
      permission: level.specifics(specific.clone()),
    })
    .resolve(&WriteArgs {
      user: sync_user().to_owned(),
    })
    .await
    {
      *has_error = true;
      log.push_str(&format!(
        "\n{}: failed to set permission in group {} | target: {target:?} | {:#}",
        colored("ERROR", Color::Red),
        bold(&user_group),
        e.error
      ))
    } else {
      log.push_str(&format!(
        "\n{}: {} user group '{}' permissions | {}: {target:?} | {}: {level} | {}: {}",
        muted("INFO"),
        colored("updated", Color::Blue),
        bold(&user_group),
        muted("target"),
        muted("level"),
        muted("specific"),
        specific.into_iter().map(|s| s.into()).collect::<Vec<&'static str>>().join(", ")
      ))
    }
  }
}

/// Expands any regex defined targets into the full list
async fn expand_user_group_permissions(
  permissions: Vec<PermissionToml>,
  all_resources: &AllResourcesById,
) -> anyhow::Result<Vec<PermissionToml>> {
  let mut expanded =
    Vec::<PermissionToml>::with_capacity(permissions.capacity());

  for permission in permissions {
    let (variant, id) = permission.target.extract_variant_id();
    if id.is_empty() {
      continue;
    }
    let matcher = Matcher::new(&id)?;
    match variant {
      ResourceTargetVariant::Build => {
        let permissions = all_resources
          .builds
          .values()
          .filter(|resource| matcher.is_match(&resource.name))
          .map(|resource| PermissionToml {
            target: ResourceTarget::Build(resource.name.clone()),
            level: permission.level,
            specific: permission.specific.clone(),
          });
        expanded.extend(permissions);
      }
      ResourceTargetVariant::Builder => {
        let permissions = all_resources
          .builders
          .values()
          .filter(|resource| matcher.is_match(&resource.name))
          .map(|resource| PermissionToml {
            target: ResourceTarget::Builder(resource.name.clone()),
            level: permission.level,
            specific: permission.specific.clone(),
          });
        expanded.extend(permissions);
      }
      ResourceTargetVariant::Deployment => {
        let permissions = all_resources
          .deployments
          .values()
          .filter(|resource| matcher.is_match(&resource.name))
          .map(|resource| PermissionToml {
            target: ResourceTarget::Deployment(resource.name.clone()),
            level: permission.level,
            specific: permission.specific.clone(),
          });
        expanded.extend(permissions);
      }
      ResourceTargetVariant::Server => {
        let permissions = all_resources
          .servers
          .values()
          .filter(|resource| matcher.is_match(&resource.name))
          .map(|resource| PermissionToml {
            target: ResourceTarget::Server(resource.name.clone()),
            level: permission.level,
            specific: permission.specific.clone(),
          });
        expanded.extend(permissions);
      }
      ResourceTargetVariant::Repo => {
        let permissions = all_resources
          .repos
          .values()
          .filter(|resource| matcher.is_match(&resource.name))
          .map(|resource| PermissionToml {
            target: ResourceTarget::Repo(resource.name.clone()),
            level: permission.level,
            specific: permission.specific.clone(),
          });
        expanded.extend(permissions);
      }
      ResourceTargetVariant::Alerter => {
        let permissions = all_resources
          .alerters
          .values()
          .filter(|resource| matcher.is_match(&resource.name))
          .map(|resource| PermissionToml {
            target: ResourceTarget::Alerter(resource.name.clone()),
            level: permission.level,
            specific: permission.specific.clone(),
          });
        expanded.extend(permissions);
      }
      ResourceTargetVariant::Procedure => {
        let permissions = all_resources
          .procedures
          .values()
          .filter(|resource| matcher.is_match(&resource.name))
          .map(|resource| PermissionToml {
            target: ResourceTarget::Procedure(resource.name.clone()),
            level: permission.level,
            specific: permission.specific.clone(),
          });
        expanded.extend(permissions);
      }
      ResourceTargetVariant::Action => {
        let permissions = all_resources
          .actions
          .values()
          .filter(|resource| matcher.is_match(&resource.name))
          .map(|resource| PermissionToml {
            target: ResourceTarget::Action(resource.name.clone()),
            level: permission.level,
            specific: permission.specific.clone(),
          });
        expanded.extend(permissions);
      }
      ResourceTargetVariant::ResourceSync => {
        let permissions = all_resources
          .syncs
          .values()
          .filter(|resource| matcher.is_match(&resource.name))
          .map(|resource| PermissionToml {
            target: ResourceTarget::ResourceSync(
              resource.name.clone(),
            ),
            level: permission.level,
            specific: permission.specific.clone(),
          });
        expanded.extend(permissions);
      }
      ResourceTargetVariant::Stack => {
        let permissions = all_resources
          .stacks
          .values()
          .filter(|resource| matcher.is_match(&resource.name))
          .map(|resource| PermissionToml {
            target: ResourceTarget::Stack(resource.name.clone()),
            level: permission.level,
            specific: permission.specific.clone(),
          });
        expanded.extend(permissions);
      }
      ResourceTargetVariant::System => {}
    }
  }

  Ok(expanded)
}

type AllDiff = IndexMap<
  ResourceTargetVariant,
  (PermissionLevelAndSpecifics, PermissionLevelAndSpecifics),
>;

fn default_permission() -> &'static PermissionLevelAndSpecifics {
  static DEFAULT_PERMISSION: OnceLock<PermissionLevelAndSpecifics> =
    OnceLock::new();
  DEFAULT_PERMISSION.get_or_init(Default::default)
}

/// diffs user_group.all
fn diff_group_all(
  original: &IndexMap<
    ResourceTargetVariant,
    PermissionLevelAndSpecifics,
  >,
  incoming: &IndexMap<
    ResourceTargetVariant,
    PermissionLevelAndSpecifics,
  >,
) -> AllDiff {
  let mut to_update = IndexMap::new();

  // need to compare both forward and backward because either hashmap could be sparse.

  // forward direction
  for (variant, permission) in incoming {
    let original_permission =
      original.get(variant).unwrap_or(default_permission());
    if permission.level != original_permission.level
      || !specific_equal(
        &original_permission.specific,
        &permission.specific,
      )
    {
      to_update.insert(
        *variant,
        (original_permission.clone(), permission.clone()),
      );
    }
  }

  // backward direction
  for (variant, permission) in original {
    let incoming_permission =
      incoming.get(variant).unwrap_or(default_permission());
    if permission.level != incoming_permission.level
      || !specific_equal(
        &incoming_permission.specific,
        &permission.specific,
      )
    {
      to_update.insert(
        *variant,
        (permission.clone(), incoming_permission.clone()),
      );
    }
  }

  to_update
}

fn specific_equal(
  a: &IndexSet<SpecificPermission>,
  b: &IndexSet<SpecificPermission>,
) -> bool {
  for item in a {
    if !b.contains(item) {
      return false;
    }
  }
  for item in b {
    if !a.contains(item) {
      return false;
    }
  }
  true
}

pub async fn convert_user_groups(
  user_groups: impl Iterator<Item = UserGroup>,
  all: &AllResourcesById,
  res: &mut Vec<(String, UserGroupToml)>,
) -> anyhow::Result<()> {
  let db = db_client();

  let usernames = find_collect(&db.users, None, None)
    .await?
    .into_iter()
    .map(|user| (user.id, user.username))
    .collect::<HashMap<_, _>>();

  for user_group in user_groups {
    // this method is admin only, but we already know user can see user group if above does not return Err
    let mut permissions = (ListUserTargetPermissions {
      user_target: UserTarget::UserGroup(user_group.id.clone()),
    })
    .resolve(&ReadArgs {
      user: User {
        admin: true,
        ..Default::default()
      },
    })
    .await
    .map_err(|e| e.error)?
    .into_iter()
    .filter(|permission| permission.level > PermissionLevel::None)
    .map(|mut permission| {
      match &mut permission.resource_target {
        ResourceTarget::Build(id) => {
          *id = all
            .builds
            .get(id)
            .map(|r| r.name.clone())
            .unwrap_or_default()
        }
        ResourceTarget::Builder(id) => {
          *id = all
            .builders
            .get(id)
            .map(|r| r.name.clone())
            .unwrap_or_default()
        }
        ResourceTarget::Deployment(id) => {
          *id = all
            .deployments
            .get(id)
            .map(|r| r.name.clone())
            .unwrap_or_default()
        }
        ResourceTarget::Server(id) => {
          *id = all
            .servers
            .get(id)
            .map(|r| r.name.clone())
            .unwrap_or_default()
        }
        ResourceTarget::Repo(id) => {
          *id = all
            .repos
            .get(id)
            .map(|r| r.name.clone())
            .unwrap_or_default()
        }
        ResourceTarget::Alerter(id) => {
          *id = all
            .alerters
            .get(id)
            .map(|r| r.name.clone())
            .unwrap_or_default()
        }
        ResourceTarget::Procedure(id) => {
          *id = all
            .procedures
            .get(id)
            .map(|r| r.name.clone())
            .unwrap_or_default()
        }
        ResourceTarget::Action(id) => {
          *id = all
            .actions
            .get(id)
            .map(|r| r.name.clone())
            .unwrap_or_default()
        }
        ResourceTarget::ResourceSync(id) => {
          *id = all
            .syncs
            .get(id)
            .map(|r| r.name.clone())
            .unwrap_or_default()
        }
        ResourceTarget::Stack(id) => {
          *id = all
            .stacks
            .get(id)
            .map(|r| r.name.clone())
            .unwrap_or_default()
        }
        ResourceTarget::System(_) => {}
      }
      PermissionToml {
        target: permission.resource_target,
        level: permission.level,
        specific: permission.specific,
      }
    })
    .collect::<Vec<_>>();

    let mut users = if user_group.everyone {
      Vec::new()
    } else {
      user_group
        .users
        .into_iter()
        .filter_map(|user_id| usernames.get(&user_id).cloned())
        .collect::<Vec<_>>()
    };

    permissions.sort_by(sort_permissions);
    users.sort();

    res.push((
      user_group.id,
      UserGroupToml {
        name: user_group.name,
        everyone: user_group.everyone,
        all: user_group.all,
        users: if user_group.everyone {
          Vec::new()
        } else {
          users
        },
        permissions,
      },
    ));
  }
  Ok(())
}
