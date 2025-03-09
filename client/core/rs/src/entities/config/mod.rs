use serde::{Deserialize, Serialize};
use typeshare::typeshare;

pub mod core;
pub mod periphery;

#[typeshare]
#[derive(
  Debug,
  Clone,
  PartialEq,
  Eq,
  Hash,
  PartialOrd,
  Ord,
  Serialize,
  Deserialize,
)]
pub struct GitProvider {
  /// The git provider domain. Default: `github.com`.
  #[serde(default = "default_git_provider")]
  pub domain: String,
  /// Whether to use https. Default: true.
  #[serde(default = "default_git_https")]
  pub https: bool,
  /// The accounts on the git provider. Required.
  #[serde(alias = "account")]
  pub accounts: Vec<ProviderAccount>,
}

fn default_git_provider() -> String {
  String::from("github.com")
}

fn default_git_https() -> bool {
  true
}

#[typeshare]
#[derive(
  Debug,
  Clone,
  PartialEq,
  Eq,
  Hash,
  PartialOrd,
  Ord,
  Serialize,
  Deserialize,
)]
pub struct DockerRegistry {
  /// The docker provider domain. Default: `docker.io`.
  #[serde(default = "default_docker_provider")]
  pub domain: String,
  /// The accounts on the registry. Required.
  #[serde(alias = "account")]
  pub accounts: Vec<ProviderAccount>,
  /// Available organizations on the registry provider.
  /// Used to push an image under an organization's repo rather than an account's repo.
  #[serde(default, alias = "organization")]
  pub organizations: Vec<String>,
}

fn default_docker_provider() -> String {
  String::from("docker.io")
}

#[typeshare]
#[derive(
  Debug,
  Clone,
  PartialEq,
  Eq,
  Hash,
  PartialOrd,
  Ord,
  Serialize,
  Deserialize,
)]
pub struct ProviderAccount {
  /// The account username. Required.
  #[serde(alias = "account")]
  pub username: String,
  /// The account access token. Required.
  #[serde(default, skip_serializing)]
  pub token: String,
}

fn empty_or_redacted(src: &str) -> String {
  if src.is_empty() {
    String::new()
  } else {
    String::from("##############")
  }
}
