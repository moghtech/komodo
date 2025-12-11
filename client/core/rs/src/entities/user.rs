use std::{collections::HashMap, sync::OnceLock};

use base64urlsafedata::Base64UrlSafeData;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;
use webauthn_rs::prelude::Passkey;

use crate::entities::{I64, MongoId};

use super::{
  ResourceTargetVariant, permission::PermissionLevelAndSpecifics,
};

#[typeshare]
pub type BytesArray = Vec<u8>;
#[typeshare(serialized_as = "BytesArray")]
pub type _Base64UrlSafeData = Base64UrlSafeData;
#[typeshare(serialized_as = "any")]
pub type _Passkey = Passkey;

#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[cfg_attr(
  feature = "mongo",
  derive(mongo_indexed::derive::MongoIndexed)
)]
#[cfg_attr(feature = "mongo", doc_index({ "config.type": 1 }))]
#[cfg_attr(feature = "mongo", sparse_doc_index({ "config.data.google_id": 1 }))]
#[cfg_attr(feature = "mongo", sparse_doc_index({ "config.data.github_id": 1 }))]
pub struct User {
  /// The Mongo ID of the User.
  /// This field is de/serialized from/to JSON as
  /// `{ "_id": { "$oid": "..." }, ...(rest of User schema) }`
  #[serde(
    default,
    rename = "_id",
    skip_serializing_if = "String::is_empty",
    with = "bson::serde_helpers::hex_string_as_object_id"
  )]
  pub id: MongoId,

  /// The globally unique username for the user.
  #[cfg_attr(feature = "mongo", unique_index)]
  pub username: String,

  /// Whether user is enabled / able to access the api.
  #[cfg_attr(feature = "mongo", index)]
  #[serde(default)]
  pub enabled: bool,

  /// Can give / take other users admin priviledges.
  #[serde(default)]
  pub super_admin: bool,

  /// Whether the user has global admin permissions.
  #[serde(default)]
  pub admin: bool,

  /// Whether the user has permission to create servers.
  #[serde(default)]
  pub create_server_permissions: bool,

  /// Whether the user has permission to create builds
  #[serde(default)]
  pub create_build_permissions: bool,

  /// The user-type specific config.
  pub config: UserConfig,

  /// TOTP 2fa credentials
  #[serde(default)]
  pub totp: UserTotpConfig,

  /// WebAuthn Passkey 2fa credentials
  #[serde(default)]
  pub passkey: UserPasskeyConfig,

  /// When the user last opened updates dropdown.
  #[serde(default)]
  pub last_update_view: I64,

  /// Recently viewed ids
  #[serde(default)]
  pub recents: HashMap<ResourceTargetVariant, Vec<String>>,

  /// Give the user elevated permissions on all resources of a certain type
  #[serde(default)]
  pub all:
    IndexMap<ResourceTargetVariant, PermissionLevelAndSpecifics>,

  #[serde(default)]
  pub updated_at: I64,
}

#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum UserConfig {
  /// User that logs in with username / password
  Local { password: String },

  /// User that logs in via Google Oauth
  Google { google_id: String, avatar: String },

  /// User that logs in via Github Oauth
  Github { github_id: String, avatar: String },

  /// User that logs in via Oidc provider
  Oidc { provider: String, user_id: String },

  /// Non-human managed user, can have it's own permissions / api keys
  Service { description: String },
}

impl Default for UserConfig {
  fn default() -> Self {
    Self::Local {
      password: String::new(),
    }
  }
}

#[typeshare]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UserTotpConfig {
  /// TOTP shared secret, encrypted
  pub secret: String,
  /// Unix timestamp in milliseconds when secret confirmed
  pub confirmed_at: I64,
  /// Hashed recovery codes.
  pub recovery_codes: Vec<String>,
}

impl UserTotpConfig {
  pub fn sanitize(&mut self) {
    self.secret.clear();
    self.recovery_codes.clear();
  }

  pub fn enrolled(&self) -> bool {
    !self.secret.is_empty()
  }
}

#[typeshare]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UserPasskeyConfig {
  pub passkey: Option<_Passkey>,
  /// Unix timestamp in milliseconds when key created
  pub created_at: I64,
}

impl UserPasskeyConfig {
  pub fn sanitize(&mut self) {
    self.passkey = None;
  }
}

impl User {
  /// Prepares user object for transport by clearing any sensitive fields
  pub fn sanitize(&mut self) {
    if let UserConfig::Local { password } = &mut self.config {
      password.clear();
    }
    self.totp.sanitize();
    self.passkey.sanitize();
  }

  /// Returns whether user is an inbuilt service user
  ///
  /// NOTE: ALSO UPDATE `frontend/src/lib/utils/is_service_user` to match
  pub fn is_service_user(user_id: &str) -> bool {
    matches!(
      user_id,
      "System"
        | "000000000000000000000000"
        | "Procedure"
        | "000000000000000000000001"
        | "Action"
        | "000000000000000000000002"
        | "Git Webhook"
        | "000000000000000000000003"
        | "Auto Redeploy"
        | "000000000000000000000004"
        | "Resource Sync"
        | "000000000000000000000005"
        | "Stack Wizard"
        | "000000000000000000000006"
        | "Build Manager"
        | "000000000000000000000007"
        | "Repo Manager"
        | "000000000000000000000008"
    )
  }
}

pub fn admin_service_user(user_id: &str) -> Option<User> {
  match user_id {
    "000000000000000000000000" | "System" => {
      system_user().to_owned().into()
    }
    "000000000000000000000001" | "Procedure" => {
      procedure_user().to_owned().into()
    }
    "000000000000000000000002" | "Action" => {
      action_user().to_owned().into()
    }
    "000000000000000000000003" | "Git Webhook" => {
      git_webhook_user().to_owned().into()
    }
    "000000000000000000000004" | "Auto Redeploy" => {
      auto_redeploy_user().to_owned().into()
    }
    "000000000000000000000005" | "Resource Sync" => {
      sync_user().to_owned().into()
    }
    "000000000000000000000006" | "Stack Wizard" => {
      stack_user().to_owned().into()
    }
    "000000000000000000000007" | "Build Manager" => {
      build_user().to_owned().into()
    }
    "000000000000000000000008" | "Repo Manager" => {
      repo_user().to_owned().into()
    }
    _ => None,
  }
}

pub fn system_user() -> &'static User {
  static SYSTEM_USER: OnceLock<User> = OnceLock::new();
  SYSTEM_USER.get_or_init(|| {
    let id_name = String::from("System");
    User {
      id: "000000000000000000000000".to_string(),
      username: id_name,
      enabled: true,
      admin: true,
      ..Default::default()
    }
  })
}

pub fn procedure_user() -> &'static User {
  static PROCEDURE_USER: OnceLock<User> = OnceLock::new();
  PROCEDURE_USER.get_or_init(|| {
    let id_name = String::from("Procedure");
    User {
      id: "000000000000000000000001".to_string(),
      username: id_name,
      enabled: true,
      admin: true,
      ..Default::default()
    }
  })
}

pub fn action_user() -> &'static User {
  static ACTION_USER: OnceLock<User> = OnceLock::new();
  ACTION_USER.get_or_init(|| {
    let id_name = String::from("Action");
    User {
      id: "000000000000000000000002".to_string(),
      username: id_name,
      enabled: true,
      admin: true,
      ..Default::default()
    }
  })
}

pub fn git_webhook_user() -> &'static User {
  static GIT_WEBHOOK_USER: OnceLock<User> = OnceLock::new();
  GIT_WEBHOOK_USER.get_or_init(|| {
    let id_name = String::from("Git Webhook");
    User {
      id: "000000000000000000000003".to_string(),
      username: id_name,
      enabled: true,
      admin: true,
      ..Default::default()
    }
  })
}

pub fn auto_redeploy_user() -> &'static User {
  static AUTO_REDEPLOY_USER: OnceLock<User> = OnceLock::new();
  AUTO_REDEPLOY_USER.get_or_init(|| {
    let id_name = String::from("Auto Redeploy");
    User {
      id: "000000000000000000000004".to_string(),
      username: id_name,
      enabled: true,
      admin: true,
      ..Default::default()
    }
  })
}

pub fn sync_user() -> &'static User {
  static SYNC_USER: OnceLock<User> = OnceLock::new();
  SYNC_USER.get_or_init(|| {
    let id_name = String::from("Resource Sync");
    User {
      id: "000000000000000000000005".to_string(),
      username: id_name,
      enabled: true,
      admin: true,
      ..Default::default()
    }
  })
}

pub fn stack_user() -> &'static User {
  static STACK_USER: OnceLock<User> = OnceLock::new();
  STACK_USER.get_or_init(|| {
    let id_name = String::from("Stack Wizard");
    User {
      id: "000000000000000000000006".to_string(),
      username: id_name,
      enabled: true,
      admin: true,
      ..Default::default()
    }
  })
}

pub fn build_user() -> &'static User {
  static BUILD_USER: OnceLock<User> = OnceLock::new();
  BUILD_USER.get_or_init(|| {
    let id_name = String::from("Build Manager");
    User {
      id: "000000000000000000000007".to_string(),
      username: id_name,
      enabled: true,
      admin: true,
      ..Default::default()
    }
  })
}

pub fn repo_user() -> &'static User {
  static REPO_USER: OnceLock<User> = OnceLock::new();
  REPO_USER.get_or_init(|| {
    let id_name = String::from("Repo Manager");
    User {
      id: "000000000000000000000008".to_string(),
      username: id_name,
      enabled: true,
      admin: true,
      ..Default::default()
    }
  })
}
