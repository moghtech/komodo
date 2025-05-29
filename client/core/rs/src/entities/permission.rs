use std::fmt::Write;

use derive_variants::EnumVariants;
use indexmap::IndexSet;
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, Display, EnumString, VariantArray};
use typeshare::typeshare;

use super::{MongoId, ResourceTarget};

/// Representation of a User or UserGroups permission on a resource.
#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(
  feature = "mongo",
  derive(mongo_indexed::derive::MongoIndexed)
)]
// To query for all permissions on user target
#[cfg_attr(feature = "mongo", doc_index({ "user_target.type": 1, "user_target.id": 1 }))]
// To query for all permissions on a resource target
#[cfg_attr(feature = "mongo", doc_index({ "resource_target.type": 1, "resource_target.id": 1 }))]
// Only one permission allowed per user / resource target
#[cfg_attr(feature = "mongo", unique_doc_index({
  "user_target.type": 1,
  "user_target.id": 1,
  "resource_target.type": 1,
  "resource_target.id": 1
}))]
pub struct Permission {
  /// The id of the permission document
  #[serde(
    default,
    rename = "_id",
    skip_serializing_if = "String::is_empty",
    with = "bson::serde_helpers::hex_string_as_object_id"
  )]
  pub id: MongoId,
  /// The target User / UserGroup
  pub user_target: UserTarget,
  /// The target resource
  pub resource_target: ResourceTarget,
  /// The permission level for the [user_target] on the [resource_target].
  #[serde(default)]
  pub level: PermissionLevel,
  /// Any specific permissions for the [user_target] on the [resource_target].
  #[serde(default)]
  pub specific: IndexSet<SpecificPermission>,
}

#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize, EnumVariants)]
#[variant_derive(
  Debug,
  Clone,
  Copy,
  Serialize,
  Deserialize,
  AsRefStr
)]
#[serde(tag = "type", content = "id")]
pub enum UserTarget {
  /// User Id
  User(String),
  /// UserGroup Id
  UserGroup(String),
}

impl UserTarget {
  pub fn extract_variant_id(self) -> (UserTargetVariant, String) {
    match self {
      UserTarget::User(id) => (UserTargetVariant::User, id),
      UserTarget::UserGroup(id) => (UserTargetVariant::UserGroup, id),
    }
  }
}

/// The levels of permission that a User or UserGroup can have on a resource.
#[typeshare]
#[derive(
  Serialize,
  Deserialize,
  Debug,
  Display,
  EnumString,
  AsRefStr,
  Hash,
  Clone,
  Copy,
  PartialEq,
  Eq,
  PartialOrd,
  Ord,
  Default,
)]
pub enum PermissionLevel {
  /// No permissions.
  #[default]
  None,
  /// Can read resource information and config
  Read,
  /// Can execute actions on the resource
  Execute,
  /// Can update the resource configuration
  Write,
}

impl Default for &PermissionLevel {
  fn default() -> Self {
    &PermissionLevel::None
  }
}

/// The specific types of permission that a User or UserGroup can have on a resource.
#[typeshare]
#[derive(
  Serialize,
  Deserialize,
  Debug,
  Display,
  EnumString,
  AsRefStr,
  VariantArray,
  Hash,
  Clone,
  Copy,
  PartialEq,
  Eq,
  PartialOrd,
  Ord,
)]
pub enum SpecificPermission {
  /// On **Server**
  ///   - Access the terminal apis
  /// On **Stack / Deployment**
  ///   - Access the container exec Apis
  Terminal,
  /// On **Server**
  ///   - Allowed to attach Stacks, Deployments, Repos, Builders to the Server
  /// On **Builder**
  ///   - Allowed to attach Builds to the Builder
  /// On **Build**
  ///   - Allowed to attach Deployments to the Build
  Attach,
  /// On **Server**
  ///   - Access the `docker inspect` apis
  /// On **Stack / Deployment**
  ///   - Access `docker inspect $container` for associated containers
  Inspect,
  /// On **Server**
  ///   - Read all container logs on the server
  /// On **Stack / Deployment**
  ///   - Read the container logs
  Logs,
  /// On **Server**
  ///   - Read all the processes on the host
  Processes,
}

impl SpecificPermission {
  fn all() -> IndexSet<SpecificPermission> {
    SpecificPermission::VARIANTS.into_iter().cloned().collect()
  }
}

#[typeshare]
#[derive(Debug, Clone, Default)]
pub struct PermissionLevelAndSpecifics {
  pub level: PermissionLevel,
  pub specific: IndexSet<SpecificPermission>,
}

impl From<PermissionLevel> for PermissionLevelAndSpecifics {
  fn from(level: PermissionLevel) -> Self {
    Self {
      level,
      specific: IndexSet::new(),
    }
  }
}

impl From<&Permission> for PermissionLevelAndSpecifics {
  fn from(value: &Permission) -> Self {
    Self {
      level: value.level,
      specific: value.specific.clone(),
    }
  }
}

impl PermissionLevel {
  /// Add all possible permissions (for use in admin case)
  pub fn all(self) -> PermissionLevelAndSpecifics {
    PermissionLevelAndSpecifics {
      level: self,
      specific: SpecificPermission::all(),
    }
  }

  pub fn specifics(
    self,
    specific: IndexSet<SpecificPermission>,
  ) -> PermissionLevelAndSpecifics {
    PermissionLevelAndSpecifics {
      level: self,
      specific,
    }
  }

  fn specific(
    self,
    specific: SpecificPermission,
  ) -> PermissionLevelAndSpecifics {
    PermissionLevelAndSpecifics {
      level: self,
      specific: [specific].into_iter().collect(),
    }
  }

  /// Operation requires Terminal permission
  pub fn terminal(self) -> PermissionLevelAndSpecifics {
    self.specific(SpecificPermission::Terminal)
  }

  /// Operation requires Attach permission
  pub fn attach(self) -> PermissionLevelAndSpecifics {
    self.specific(SpecificPermission::Attach)
  }

  /// Operation requires Inspect permission
  pub fn inspect(self) -> PermissionLevelAndSpecifics {
    self.specific(SpecificPermission::Inspect)
  }

  /// Operation requires Logs permission
  pub fn logs(self) -> PermissionLevelAndSpecifics {
    self.specific(SpecificPermission::Logs)
  }

  /// Operation requires Processes permission
  pub fn processes(self) -> PermissionLevelAndSpecifics {
    self.specific(SpecificPermission::Processes)
  }
}

impl PermissionLevelAndSpecifics {
  /// Returns true when self.level >= other.level,
  /// and has all required specific permissions.
  pub fn fulfills(
    &self,
    other: &PermissionLevelAndSpecifics,
  ) -> bool {
    if self.level < other.level {
      return false;
    }
    for specific in other.specific.iter() {
      if !self.specific.contains(specific) {
        return false;
      }
    }
    true
  }

  /// Returns true when self has all required specific permissions.
  pub fn fulfills_specific(
    &self,
    specifics: &IndexSet<SpecificPermission>,
  ) -> bool {
    for specific in specifics.iter() {
      if !self.specific.contains(specific) {
        return false;
      }
    }
    true
  }

  pub fn specifics_for_log(&self) -> String {
    let mut res = String::new();
    for specific in self.specific.iter() {
      write!(&mut res, ", {specific}").unwrap();
    }
    res
  }

  pub fn specifics(
    mut self,
    specific: IndexSet<SpecificPermission>,
  ) -> PermissionLevelAndSpecifics {
    self.specific = specific;
    self
  }

  fn specific(
    mut self,
    specific: SpecificPermission,
  ) -> PermissionLevelAndSpecifics {
    self.specific.insert(specific);
    PermissionLevelAndSpecifics {
      level: self.level,
      specific: self.specific,
    }
  }

  /// Operation requires Terminal permission
  pub fn terminal(self) -> PermissionLevelAndSpecifics {
    self.specific(SpecificPermission::Terminal)
  }

  /// Operation requires Attach permission
  pub fn attach(self) -> PermissionLevelAndSpecifics {
    self.specific(SpecificPermission::Attach)
  }

  /// Operation requires Inspect permission
  pub fn inspect(self) -> PermissionLevelAndSpecifics {
    self.specific(SpecificPermission::Inspect)
  }

  /// Operation requires Logs permission
  pub fn logs(self) -> PermissionLevelAndSpecifics {
    self.specific(SpecificPermission::Logs)
  }

  /// Operation requires Processes permission
  pub fn processes(self) -> PermissionLevelAndSpecifics {
    self.specific(SpecificPermission::Processes)
  }
}
