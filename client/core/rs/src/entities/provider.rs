use partial_derive2::Partial;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use super::MongoId;

#[typeshare(serialized_as = "Partial<GitProviderAccount>")]
pub type _PartialGitProviderAccount = PartialGitProviderAccount;

/// Configuration to access private git repos from various git providers.
/// Note. Cannot create two accounts with the same domain and username.
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Default, Partial)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[partial_derive(Serialize, Deserialize, Debug, Clone, Default)]
#[diff_derive(Serialize, Deserialize, Debug, Clone, Default)]
#[partial(skip_serializing_none, from, diff)]
#[cfg_attr(
  feature = "mongo",
  derive(mongo_indexed::derive::MongoIndexed)
)]
#[cfg_attr(feature = "mongo", unique_doc_index({ "domain": 1, "username": 1 }))]
pub struct GitProviderAccount {
  /// The Mongo ID of the git provider account.
  /// This field is de/serialized from/to JSON as
  /// `{ "_id": { "$oid": "..." }, ...(rest of serialized User) }`
  #[serde(
    default,
    rename = "_id",
    skip_serializing_if = "String::is_empty",
    with = "bson::serde_helpers::hex_string_as_object_id"
  )]
  pub id: MongoId,
  /// The domain of the provider.
  ///
  /// For git, this cannot include the protocol eg 'http://',
  /// which is controlled with 'https' field.
  #[cfg_attr(feature = "mongo", index)]
  #[serde(default = "default_git_domain")]
  #[partial_default(default_git_domain())]
  pub domain: String,
  /// Whether git provider is accessed over http or https.
  #[serde(default = "default_https")]
  #[partial_default(default_https())]
  pub https: bool,
  /// The account username
  #[cfg_attr(feature = "mongo", index)]
  #[serde(default)]
  pub username: String,
  /// The token in plain text on the db.
  /// If the database / host can be accessed this is insecure.
  #[serde(default)]
  pub token: String,
}

fn default_git_domain() -> String {
  String::from("github.com")
}

fn default_https() -> bool {
  true
}

#[cfg(feature = "utoipa")]
impl utoipa::PartialSchema for PartialGitProviderAccount {
  fn schema()
  -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
    utoipa::schema!(#[inline] std::collections::HashMap<String, serde_json::Value>).into()
  }
}

#[cfg(feature = "utoipa")]
impl utoipa::ToSchema for PartialGitProviderAccount {}

#[typeshare(serialized_as = "Partial<DockerRegistryAccount>")]
pub type _PartialDockerRegistryAccount = PartialDockerRegistryAccount;

/// Configuration to access private image repositories on various registries.
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Default, Partial)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[partial_derive(Serialize, Deserialize, Debug, Clone, Default)]
#[diff_derive(Serialize, Deserialize, Debug, Clone, Default)]
#[partial(skip_serializing_none, from, diff)]
#[cfg_attr(
  feature = "mongo",
  derive(mongo_indexed::derive::MongoIndexed)
)]
#[cfg_attr(feature = "mongo", unique_doc_index({ "domain": 1, "username": 1 }))]
pub struct DockerRegistryAccount {
  /// The Mongo ID of the docker registry account.
  /// This field is de/serialized from/to JSON as
  /// `{ "_id": { "$oid": "..." }, ...(rest of DockerRegistryAccount) }`
  #[serde(
    default,
    rename = "_id",
    skip_serializing_if = "String::is_empty",
    with = "bson::serde_helpers::hex_string_as_object_id"
  )]
  pub id: MongoId,
  /// The domain of the provider.
  ///
  /// For docker registry, this can include 'http://...',
  /// however this is not recommended and won't work unless "insecure registries" are enabled
  /// on your hosts. See <https://docs.docker.com/reference/cli/dockerd/#insecure-registries>.
  #[cfg_attr(feature = "mongo", index)]
  #[serde(default = "default_registry_domain")]
  #[partial_default(default_registry_domain())]
  pub domain: String,
  /// The account username
  #[cfg_attr(feature = "mongo", index)]
  #[serde(default)]
  pub username: String,
  /// The token in plain text on the db.
  /// If the database / host can be accessed this is insecure.
  #[serde(default)]
  pub token: String,
}

fn default_registry_domain() -> String {
  String::from("docker.io")
}

#[cfg(feature = "utoipa")]
impl utoipa::PartialSchema for PartialDockerRegistryAccount {
  fn schema()
  -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
    utoipa::schema!(#[inline] std::collections::HashMap<String, serde_json::Value>).into()
  }
}

#[cfg(feature = "utoipa")]
impl utoipa::ToSchema for PartialDockerRegistryAccount {}
