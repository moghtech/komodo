use std::{
  path::{Path, PathBuf},
  str::FromStr,
};

use anyhow::Context;
use async_timing_util::unix_timestamp_ms;
use build::ImageRegistryConfig;
use clap::Parser;
use derive_empty_traits::EmptyTraits;
use derive_variants::{EnumVariants, ExtractVariant};
use serde::{
  Deserialize, Serialize,
  de::{Visitor, value::MapAccessDeserializer},
};
use serror::Serror;
use strum::{AsRefStr, Display, EnumString};
use typeshare::typeshare;

use crate::{
  deserializers::file_contents_deserializer,
  parsers::parse_key_value_list,
};

/// Subtypes of [Action][action::Action].
pub mod action;
/// Subtypes of [Alert][alert::Alert].
pub mod alert;
/// Subtypes of [Alerter][alerter::Alerter].
pub mod alerter;
/// Subtypes of [ApiKey][api_key::ApiKey].
pub mod api_key;
/// Subtypes of [Build][build::Build].
pub mod build;
/// Subtypes of [Builder][builder::Builder].
pub mod builder;
/// [core config][config::core] and [periphery config][config::periphery]
pub mod config;
/// Subtypes of [Deployment][deployment::Deployment].
pub mod deployment;
/// Networks, Images, Containers.
pub mod docker;
/// Subtypes of [LogConfig][logger::LogConfig].
pub mod logger;
/// Subtypes of [Permission][permission::Permission].
pub mod permission;
/// Subtypes of [Procedure][procedure::Procedure].
pub mod procedure;
/// Subtypes of [GitProviderAccount][provider::GitProviderAccount] and [DockerRegistryAccount][provider::DockerRegistryAccount]
pub mod provider;
/// Subtypes of [Repo][repo::Repo].
pub mod repo;
/// Subtypes of [Resource][resource::Resource].
pub mod resource;
/// Subtypes of [Schedule][schedule::Schedule]
pub mod schedule;
/// Subtypes of [Server][server::Server].
pub mod server;
/// Subtypes of [Stack][stack::Stack]
pub mod stack;
/// Subtypes for server stats reporting.
pub mod stats;
/// Subtypes of [ResourceSync][sync::ResourceSync]
pub mod sync;
/// Subtypes of [Tag][tag::Tag].
pub mod tag;
/// Subtypes of [ResourcesToml][toml::ResourcesToml].
pub mod toml;
/// Subtypes of [Update][update::Update].
pub mod update;
/// Subtypes of [User][user::User].
pub mod user;
/// Subtypes of [UserGroup][user_group::UserGroup].
pub mod user_group;
/// Subtypes of [Variable][variable::Variable]
pub mod variable;

#[typeshare(serialized_as = "number")]
pub type I64 = i64;
#[typeshare(serialized_as = "number")]
pub type U64 = u64;
#[typeshare(serialized_as = "number")]
pub type Usize = usize;
#[typeshare(serialized_as = "any")]
pub type MongoDocument = bson::Document;
#[typeshare(serialized_as = "any")]
pub type JsonValue = serde_json::Value;
#[typeshare(serialized_as = "MongoIdObj")]
pub type MongoId = String;
#[typeshare(serialized_as = "__Serror")]
pub type _Serror = Serror;

/// Represents an empty json object: `{}`
#[typeshare]
#[derive(
  Debug,
  Clone,
  Default,
  PartialEq,
  Serialize,
  Deserialize,
  Parser,
  EmptyTraits,
)]
pub struct NoData {}

pub trait MergePartial: Sized {
  type Partial;
  fn merge_partial(self, partial: Self::Partial) -> Self;
}

pub fn all_logs_success(logs: &[update::Log]) -> bool {
  for log in logs {
    if !log.success {
      return false;
    }
  }
  true
}

pub fn optional_string(string: impl Into<String>) -> Option<String> {
  let string = string.into();
  if string.is_empty() {
    None
  } else {
    Some(string)
  }
}

pub fn get_image_name(
  build::Build {
    name,
    config:
      build::BuildConfig {
        image_name,
        image_registry:
          ImageRegistryConfig {
            domain,
            account,
            organization,
          },
        ..
      },
    ..
  }: &build::Build,
) -> anyhow::Result<String> {
  let name = if image_name.is_empty() {
    name
  } else {
    image_name
  };
  let name = match (
    !domain.is_empty(),
    !organization.is_empty(),
    !account.is_empty(),
  ) {
    // If organization and account provided, name under organization.
    (true, true, true) => {
      format!("{domain}/{}/{name}", organization)
    }
    // Just domain / account provided
    (true, false, true) => format!("{domain}/{account}/{name}"),
    // Otherwise, just use name
    _ => name.to_string(),
  };
  Ok(name)
}

pub fn to_general_name(name: &str) -> String {
  name.trim().replace('\n', "_").to_string()
}

pub fn to_path_compatible_name(name: &str) -> String {
  name.trim().replace([' ', '\n'], "_").to_string()
}

/// Enforce common container naming rules.
/// [a-zA-Z0-9_.-]
pub fn to_container_compatible_name(name: &str) -> String {
  name.trim().replace([' ', ',', '\n', '&'], "_").to_string()
}

/// Enforce common docker naming rules, such as only lowercase, and no '.'.
/// These apply to:
///   - Stacks (docker project name)
///   - Builds (docker image name)
///   - Networks
///   - Volumes
pub fn to_docker_compatible_name(name: &str) -> String {
  name
    .to_lowercase()
    .replace([' ', '.', ',', '\n', '&'], "_")
    .trim()
    .to_string()
}

/// Unix timestamp in milliseconds as i64
pub fn komodo_timestamp() -> i64 {
  unix_timestamp_ms() as i64
}

#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MongoIdObj {
  #[serde(rename = "$oid")]
  pub oid: String,
}

#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct __Serror {
  pub error: String,
  pub trace: Vec<String>,
}

#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq,
)]
pub struct SystemCommand {
  #[serde(default)]
  pub path: String,
  #[serde(default, deserialize_with = "file_contents_deserializer")]
  pub command: String,
}

impl SystemCommand {
  pub fn command(&self) -> Option<String> {
    if self.is_none() {
      None
    } else {
      Some(format!("cd {} && {}", self.path, self.command))
    }
  }

  pub fn into_option(self) -> Option<SystemCommand> {
    if self.is_none() { None } else { Some(self) }
  }

  pub fn is_none(&self) -> bool {
    self.command.is_empty()
  }
}

#[typeshare]
#[derive(Serialize, Debug, Clone, Copy, Default, PartialEq)]
pub struct Version {
  pub major: i32,
  pub minor: i32,
  pub patch: i32,
}

impl<'de> Deserialize<'de> for Version {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    #[derive(Deserialize)]
    struct VersionInner {
      major: i32,
      minor: i32,
      patch: i32,
    }

    impl From<VersionInner> for Version {
      fn from(
        VersionInner {
          major,
          minor,
          patch,
        }: VersionInner,
      ) -> Self {
        Version {
          major,
          minor,
          patch,
        }
      }
    }

    struct VersionVisitor;

    impl<'de> Visitor<'de> for VersionVisitor {
      type Value = Version;
      fn expecting(
        &self,
        formatter: &mut std::fmt::Formatter,
      ) -> std::fmt::Result {
        write!(
          formatter,
          "version string or object | example: '0.2.4' or {{ \"major\": 0, \"minor\": 2, \"patch\": 4, }}"
        )
      }

      fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
      where
        E: serde::de::Error,
      {
        v.try_into()
          .map_err(|e| serde::de::Error::custom(format!("{e:#}")))
      }

      fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
      where
        A: serde::de::MapAccess<'de>,
      {
        Ok(
          VersionInner::deserialize(MapAccessDeserializer::new(map))?
            .into(),
        )
      }
    }

    deserializer.deserialize_any(VersionVisitor)
  }
}

impl std::fmt::Display for Version {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_fmt(format_args!(
      "{}.{}.{}",
      self.major, self.minor, self.patch
    ))
  }
}

impl TryFrom<&str> for Version {
  type Error = anyhow::Error;

  fn try_from(value: &str) -> Result<Self, Self::Error> {
    let mut split = value.split('.');
    let major = split
      .next()
      .context("must provide at least major version")?
      .parse::<i32>()
      .context("major version must be integer")?;
    let minor = split
      .next()
      .map(|minor| minor.parse::<i32>())
      .transpose()
      .context("minor version must be integer")?
      .unwrap_or_default();
    let patch = split
      .next()
      .map(|patch| patch.parse::<i32>())
      .transpose()
      .context("patch version must be integer")?
      .unwrap_or_default();
    Ok(Version {
      major,
      minor,
      patch,
    })
  }
}

impl Version {
  pub fn increment(&mut self) {
    self.patch += 1;
  }

  pub fn is_none(&self) -> bool {
    self.major == 0 && self.minor == 0 && self.patch == 0
  }
}

#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize,
)]
pub struct EnvironmentVar {
  pub variable: String,
  pub value: String,
}

pub fn environment_vars_from_str(
  input: &str,
) -> anyhow::Result<Vec<EnvironmentVar>> {
  parse_key_value_list(input).map(|list| {
    list
      .into_iter()
      .map(|(variable, value)| EnvironmentVar { variable, value })
      .collect()
  })
}

#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatestCommit {
  pub hash: String,
  pub message: String,
}

#[typeshare]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FileContents {
  /// The path of the file on the host
  pub path: String,
  /// The contents of the file
  pub contents: String,
}

/// Represents a scheduled maintenance window
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct MaintenanceWindow {
  /// Name for the maintenance window (required)
  pub name: String,
  /// Description of what maintenance is performed (optional)
  #[serde(default)]
  pub description: String,
  /// The type of maintenance schedule:
  ///   - Daily (default)
  ///   - Weekly
  ///   - OneTime
  #[serde(default)]
  pub schedule_type: MaintenanceScheduleType,
  /// For Weekly schedules: Specify the day of the week (Monday, Tuesday, etc.)
  #[serde(default)]
  pub day_of_week: String,
  /// For OneTime window: ISO 8601 date format (YYYY-MM-DD)
  #[serde(default)]
  pub date: String,
  /// Start hour in 24-hour format (0-23) (optional, defaults to 0)
  #[serde(default)]
  pub hour: u8,
  /// Start minute (0-59) (optional, defaults to 0)
  #[serde(default)]
  pub minute: u8,
  /// Duration of the maintenance window in minutes (required)
  pub duration_minutes: u32,
  /// Timezone for maintenance window specificiation.
  /// If empty, will use Core timezone.
  #[serde(default)]
  pub timezone: String,
  /// Whether this maintenance window is currently enabled
  #[serde(default = "default_enabled")]
  pub enabled: bool,
}

fn default_enabled() -> bool {
  true
}

#[typeshare]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct CloneArgs {
  /// Resource name (eg Build name, Repo name)
  pub name: String,
  /// Git provider domain. Default: `github.com`
  pub provider: String,
  /// Use https (vs http).
  pub https: bool,
  /// Full repo identifier. {namespace}/{repo_name}
  pub repo: Option<String>,
  /// Git Branch. Default: `main`
  pub branch: String,
  /// Specific commit hash. Optional
  pub commit: Option<String>,
  /// Use PERIPHERY_BUILD_DIR as the parent folder for the clone.
  pub is_build: bool,
  /// The clone destination path
  pub destination: Option<String>,
  /// Command to run after the repo has been cloned
  pub on_clone: Option<SystemCommand>,
  /// Command to run after the repo has been pulled
  pub on_pull: Option<SystemCommand>,
  /// Configure the account used to access repo (if private)
  pub account: Option<String>,
}

impl CloneArgs {
  pub fn path(&self, root_repo_dir: &Path) -> PathBuf {
    match &self.destination {
      Some(destination) => root_repo_dir
        .join(to_path_compatible_name(&self.name))
        .join(destination),
      None => root_repo_dir.join(to_path_compatible_name(&self.name)),
    }
    .components()
    .collect()
  }

  pub fn remote_url(
    &self,
    access_token: Option<&str>,
  ) -> anyhow::Result<String> {
    let access_token_at = match &access_token {
      Some(token) => format!("token:{token}@"),
      None => String::new(),
    };
    let protocol = if self.https { "https" } else { "http" };
    let repo = self
      .repo
      .as_ref()
      .context("resource has no repo attached")?;
    Ok(format!(
      "{protocol}://{access_token_at}{}/{repo}",
      self.provider
    ))
  }

  pub fn unique_path(
    &self,
    repo_dir: &Path,
  ) -> anyhow::Result<PathBuf> {
    let repo = self
      .repo
      .as_ref()
      .context("resource has no repo attached")?;
    let res = repo_dir
      .join(self.provider.replace('/', "-"))
      .join(repo.replace('/', "-"))
      .join(self.branch.replace('/', "-"))
      .join(self.commit.as_deref().unwrap_or("latest"))
      .components()
      .collect();
    Ok(res)
  }
}

impl From<&self::build::Build> for CloneArgs {
  fn from(build: &self::build::Build) -> CloneArgs {
    CloneArgs {
      name: build.name.clone(),
      provider: optional_string(&build.config.git_provider)
        .unwrap_or_else(|| String::from("github.com")),
      repo: optional_string(&build.config.repo),
      branch: optional_string(&build.config.branch)
        .unwrap_or_else(|| String::from("main")),
      commit: optional_string(&build.config.commit),
      is_build: true,
      destination: None,
      on_clone: None,
      on_pull: None,
      https: build.config.git_https,
      account: optional_string(&build.config.git_account),
    }
  }
}

impl From<&self::repo::Repo> for CloneArgs {
  fn from(repo: &self::repo::Repo) -> CloneArgs {
    CloneArgs {
      name: repo.name.clone(),
      provider: optional_string(&repo.config.git_provider)
        .unwrap_or_else(|| String::from("github.com")),
      repo: optional_string(&repo.config.repo),
      branch: optional_string(&repo.config.branch)
        .unwrap_or_else(|| String::from("main")),
      commit: optional_string(&repo.config.commit),
      is_build: false,
      destination: optional_string(&repo.config.path),
      on_clone: repo.config.on_clone.clone().into_option(),
      on_pull: repo.config.on_pull.clone().into_option(),
      https: repo.config.git_https,
      account: optional_string(&repo.config.git_account),
    }
  }
}

impl From<&self::sync::ResourceSync> for CloneArgs {
  fn from(sync: &self::sync::ResourceSync) -> Self {
    CloneArgs {
      name: sync.name.clone(),
      provider: optional_string(&sync.config.git_provider)
        .unwrap_or_else(|| String::from("github.com")),
      repo: optional_string(&sync.config.repo),
      branch: optional_string(&sync.config.branch)
        .unwrap_or_else(|| String::from("main")),
      commit: optional_string(&sync.config.commit),
      is_build: false,
      destination: None,
      on_clone: None,
      on_pull: None,
      https: sync.config.git_https,
      account: optional_string(&sync.config.git_account),
    }
  }
}

impl From<&self::stack::Stack> for CloneArgs {
  fn from(stack: &self::stack::Stack) -> Self {
    CloneArgs {
      name: stack.name.clone(),
      provider: optional_string(&stack.config.git_provider)
        .unwrap_or_else(|| String::from("github.com")),
      repo: optional_string(&stack.config.repo),
      branch: optional_string(&stack.config.branch)
        .unwrap_or_else(|| String::from("main")),
      commit: optional_string(&stack.config.commit),
      is_build: false,
      destination: optional_string(&stack.config.clone_path),
      on_clone: None,
      on_pull: None,
      https: stack.config.git_https,
      account: optional_string(&stack.config.git_account),
    }
  }
}

#[typeshare]
#[derive(
  Serialize,
  Deserialize,
  Debug,
  Display,
  EnumString,
  PartialEq,
  Hash,
  Eq,
  Clone,
  Copy,
  Default,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum Timelength {
  #[serde(rename = "1-sec")]
  #[strum(serialize = "1-sec")]
  OneSecond,
  #[serde(rename = "5-sec")]
  #[strum(serialize = "5-sec")]
  FiveSeconds,
  #[serde(rename = "10-sec")]
  #[strum(serialize = "10-sec")]
  TenSeconds,
  #[serde(rename = "15-sec")]
  #[strum(serialize = "15-sec")]
  FifteenSeconds,
  #[serde(rename = "30-sec")]
  #[strum(serialize = "30-sec")]
  ThirtySeconds,
  #[default]
  #[serde(rename = "1-min")]
  #[strum(serialize = "1-min")]
  OneMinute,
  #[serde(rename = "2-min")]
  #[strum(serialize = "2-min")]
  TwoMinutes,
  #[serde(rename = "5-min")]
  #[strum(serialize = "5-min")]
  FiveMinutes,
  #[serde(rename = "10-min")]
  #[strum(serialize = "10-min")]
  TenMinutes,
  #[serde(rename = "15-min")]
  #[strum(serialize = "15-min")]
  FifteenMinutes,
  #[serde(rename = "30-min")]
  #[strum(serialize = "30-min")]
  ThirtyMinutes,
  #[serde(rename = "1-hr")]
  #[strum(serialize = "1-hr")]
  OneHour,
  #[serde(rename = "2-hr")]
  #[strum(serialize = "2-hr")]
  TwoHours,
  #[serde(rename = "6-hr")]
  #[strum(serialize = "6-hr")]
  SixHours,
  #[serde(rename = "8-hr")]
  #[strum(serialize = "8-hr")]
  EightHours,
  #[serde(rename = "12-hr")]
  #[strum(serialize = "12-hr")]
  TwelveHours,
  #[serde(rename = "1-day")]
  #[strum(serialize = "1-day")]
  OneDay,
  #[serde(rename = "3-day")]
  #[strum(serialize = "3-day")]
  ThreeDay,
  #[serde(rename = "1-wk")]
  #[strum(serialize = "1-wk")]
  OneWeek,
  #[serde(rename = "2-wk")]
  #[strum(serialize = "2-wk")]
  TwoWeeks,
  #[serde(rename = "30-day")]
  #[strum(serialize = "30-day")]
  ThirtyDays,
}

impl TryInto<async_timing_util::Timelength> for Timelength {
  type Error = anyhow::Error;
  fn try_into(
    self,
  ) -> Result<async_timing_util::Timelength, Self::Error> {
    async_timing_util::Timelength::from_str(&self.to_string())
      .context("failed to parse timelength?")
  }
}

/// Days of the week
#[typeshare]
#[derive(
  Debug,
  Clone,
  Copy,
  PartialEq,
  Eq,
  Default,
  EnumString,
  Serialize,
  Deserialize,
)]
pub enum DayOfWeek {
  #[default]
  #[serde(alias = "monday", alias = "Mon", alias = "mon")]
  #[strum(serialize = "monday", serialize = "Mon", serialize = "mon")]
  Monday,
  #[serde(alias = "tuesday", alias = "Tue", alias = "tue")]
  #[strum(
    serialize = "tuesday",
    serialize = "Tue",
    serialize = "tue"
  )]
  Tuesday,
  #[serde(alias = "wednesday", alias = "Wed", alias = "wed")]
  #[strum(
    serialize = "wednesday",
    serialize = "Wed",
    serialize = "wed"
  )]
  Wednesday,
  #[serde(alias = "thursday", alias = "Thurs", alias = "thurs")]
  #[strum(
    serialize = "thursday",
    serialize = "Thurs",
    serialize = "thurs"
  )]
  Thursday,
  #[serde(alias = "friday", alias = "Fri", alias = "fri")]
  #[strum(serialize = "friday", serialize = "Fri", serialize = "fri")]
  Friday,
  #[serde(alias = "saturday", alias = "Sat", alias = "sat")]
  #[strum(
    serialize = "saturday",
    serialize = "Sat",
    serialize = "sat"
  )]
  Saturday,
  #[serde(alias = "sunday", alias = "Sun", alias = "sun")]
  #[strum(serialize = "sunday", serialize = "Sun", serialize = "sun")]
  Sunday,
}

/// Types of maintenance schedules
#[typeshare]
#[derive(
  Debug,
  Clone,
  Copy,
  PartialEq,
  Default,
  EnumString,
  Serialize,
  Deserialize,
)]
pub enum MaintenanceScheduleType {
  /// Daily at the specified time
  #[default]
  Daily,
  /// Weekly on the specified day and time
  Weekly,
  /// One-time maintenance on a specific date and time
  OneTime, // ISO 8601 date format (YYYY-MM-DD)
}

/// One representative IANA zone for each distinct base UTC offset in the tz database.
/// https://en.wikipedia.org/wiki/List_of_tz_database_time_zones.
///
/// The `serde`/`strum` renames ensure the canonical identifier is used
/// when serializing or parsing from a string such as `"Etc/UTC"`.
#[typeshare]
#[derive(
  Debug,
  Clone,
  Copy,
  PartialEq,
  Default,
  EnumString,
  Serialize,
  Deserialize,
)]
pub enum IanaTimezone {
  /// UTC−12:00
  #[serde(rename = "Etc/GMT+12")]
  #[strum(serialize = "Etc/GMT+12")]
  EtcGmtMinus12,

  /// UTC−11:00
  #[serde(rename = "Pacific/Pago_Pago")]
  #[strum(serialize = "Pacific/Pago_Pago")]
  PacificPagoPago,

  /// UTC−10:00
  #[serde(rename = "Pacific/Honolulu")]
  #[strum(serialize = "Pacific/Honolulu")]
  PacificHonolulu,

  /// UTC−09:30
  #[serde(rename = "Pacific/Marquesas")]
  #[strum(serialize = "Pacific/Marquesas")]
  PacificMarquesas,

  /// UTC−09:00
  #[serde(rename = "America/Anchorage")]
  #[strum(serialize = "America/Anchorage")]
  AmericaAnchorage,

  /// UTC−08:00
  #[serde(rename = "America/Los_Angeles")]
  #[strum(serialize = "America/Los_Angeles")]
  AmericaLosAngeles,

  /// UTC−07:00
  #[serde(rename = "America/Denver")]
  #[strum(serialize = "America/Denver")]
  AmericaDenver,

  /// UTC−06:00
  #[serde(rename = "America/Chicago")]
  #[strum(serialize = "America/Chicago")]
  AmericaChicago,

  /// UTC−05:00
  #[serde(rename = "America/New_York")]
  #[strum(serialize = "America/New_York")]
  AmericaNewYork,

  /// UTC−04:00
  #[serde(rename = "America/Halifax")]
  #[strum(serialize = "America/Halifax")]
  AmericaHalifax,

  /// UTC−03:30
  #[serde(rename = "America/St_Johns")]
  #[strum(serialize = "America/St_Johns")]
  AmericaStJohns,

  /// UTC−03:00
  #[serde(rename = "America/Sao_Paulo")]
  #[strum(serialize = "America/Sao_Paulo")]
  AmericaSaoPaulo,

  /// UTC−02:00
  #[serde(rename = "America/Noronha")]
  #[strum(serialize = "America/Noronha")]
  AmericaNoronha,

  /// UTC−01:00
  #[serde(rename = "Atlantic/Azores")]
  #[strum(serialize = "Atlantic/Azores")]
  AtlanticAzores,

  /// UTC±00:00
  #[default]
  #[serde(rename = "Etc/UTC")]
  #[strum(serialize = "Etc/UTC")]
  EtcUtc,

  /// UTC+01:00
  #[serde(rename = "Europe/Berlin")]
  #[strum(serialize = "Europe/Berlin")]
  EuropeBerlin,

  /// UTC+02:00
  #[serde(rename = "Europe/Bucharest")]
  #[strum(serialize = "Europe/Bucharest")]
  EuropeBucharest,

  /// UTC+03:00
  #[serde(rename = "Europe/Moscow")]
  #[strum(serialize = "Europe/Moscow")]
  EuropeMoscow,

  /// UTC+03:30
  #[serde(rename = "Asia/Tehran")]
  #[strum(serialize = "Asia/Tehran")]
  AsiaTehran,

  /// UTC+04:00
  #[serde(rename = "Asia/Dubai")]
  #[strum(serialize = "Asia/Dubai")]
  AsiaDubai,

  /// UTC+04:30
  #[serde(rename = "Asia/Kabul")]
  #[strum(serialize = "Asia/Kabul")]
  AsiaKabul,

  /// UTC+05:00
  #[serde(rename = "Asia/Karachi")]
  #[strum(serialize = "Asia/Karachi")]
  AsiaKarachi,

  /// UTC+05:30
  #[serde(rename = "Asia/Kolkata")]
  #[strum(serialize = "Asia/Kolkata")]
  AsiaKolkata,

  /// UTC+05:45
  #[serde(rename = "Asia/Kathmandu")]
  #[strum(serialize = "Asia/Kathmandu")]
  AsiaKathmandu,

  /// UTC+06:00
  #[serde(rename = "Asia/Dhaka")]
  #[strum(serialize = "Asia/Dhaka")]
  AsiaDhaka,

  /// UTC+06:30
  #[serde(rename = "Asia/Yangon")]
  #[strum(serialize = "Asia/Yangon")]
  AsiaYangon,

  /// UTC+07:00
  #[serde(rename = "Asia/Bangkok")]
  #[strum(serialize = "Asia/Bangkok")]
  AsiaBangkok,

  /// UTC+08:00
  #[serde(rename = "Asia/Shanghai")]
  #[strum(serialize = "Asia/Shanghai")]
  AsiaShanghai,

  /// UTC+08:45
  #[serde(rename = "Australia/Eucla")]
  #[strum(serialize = "Australia/Eucla")]
  AustraliaEucla,

  /// UTC+09:00
  #[serde(rename = "Asia/Tokyo")]
  #[strum(serialize = "Asia/Tokyo")]
  AsiaTokyo,

  /// UTC+09:30
  #[serde(rename = "Australia/Adelaide")]
  #[strum(serialize = "Australia/Adelaide")]
  AustraliaAdelaide,

  /// UTC+10:00
  #[serde(rename = "Australia/Sydney")]
  #[strum(serialize = "Australia/Sydney")]
  AustraliaSydney,

  /// UTC+10:30
  #[serde(rename = "Australia/Lord_Howe")]
  #[strum(serialize = "Australia/Lord_Howe")]
  AustraliaLordHowe,

  /// UTC+11:00
  #[serde(rename = "Pacific/Port_Moresby")]
  #[strum(serialize = "Pacific/Port_Moresby")]
  PacificPortMoresby,

  /// UTC+12:00
  #[serde(rename = "Pacific/Auckland")]
  #[strum(serialize = "Pacific/Auckland")]
  PacificAuckland,

  /// UTC+12:45
  #[serde(rename = "Pacific/Chatham")]
  #[strum(serialize = "Pacific/Chatham")]
  PacificChatham,

  /// UTC+13:00
  #[serde(rename = "Pacific/Tongatapu")]
  #[strum(serialize = "Pacific/Tongatapu")]
  PacificTongatapu,

  /// UTC+14:00
  #[serde(rename = "Pacific/Kiritimati")]
  #[strum(serialize = "Pacific/Kiritimati")]
  PacificKiritimati,
}

#[typeshare]
#[derive(
  Debug,
  Clone,
  Copy,
  PartialEq,
  Eq,
  Hash,
  Serialize,
  Deserialize,
  Default,
  Display,
  EnumString,
  AsRefStr,
)]
pub enum Operation {
  // do nothing
  #[default]
  None,

  // server
  CreateServer,
  UpdateServer,
  DeleteServer,
  RenameServer,
  StartContainer,
  RestartContainer,
  PauseContainer,
  UnpauseContainer,
  StopContainer,
  DestroyContainer,
  StartAllContainers,
  RestartAllContainers,
  PauseAllContainers,
  UnpauseAllContainers,
  StopAllContainers,
  PruneContainers,
  CreateNetwork,
  DeleteNetwork,
  PruneNetworks,
  DeleteImage,
  PruneImages,
  DeleteVolume,
  PruneVolumes,
  PruneDockerBuilders,
  PruneBuildx,
  PruneSystem,

  // stack
  CreateStack,
  UpdateStack,
  RenameStack,
  DeleteStack,
  WriteStackContents,
  RefreshStackCache,
  PullStack,
  DeployStack,
  StartStack,
  RestartStack,
  PauseStack,
  UnpauseStack,
  StopStack,
  DestroyStack,

  // stack (service)
  DeployStackService,
  PullStackService,
  StartStackService,
  RestartStackService,
  PauseStackService,
  UnpauseStackService,
  StopStackService,
  DestroyStackService,

  // deployment
  CreateDeployment,
  UpdateDeployment,
  RenameDeployment,
  DeleteDeployment,
  Deploy,
  PullDeployment,
  StartDeployment,
  RestartDeployment,
  PauseDeployment,
  UnpauseDeployment,
  StopDeployment,
  DestroyDeployment,

  // build
  CreateBuild,
  UpdateBuild,
  RenameBuild,
  DeleteBuild,
  RunBuild,
  CancelBuild,
  WriteDockerfile,

  // repo
  CreateRepo,
  UpdateRepo,
  RenameRepo,
  DeleteRepo,
  CloneRepo,
  PullRepo,
  BuildRepo,
  CancelRepoBuild,

  // procedure
  CreateProcedure,
  UpdateProcedure,
  RenameProcedure,
  DeleteProcedure,
  RunProcedure,

  // action
  CreateAction,
  UpdateAction,
  RenameAction,
  DeleteAction,
  RunAction,

  // builder
  CreateBuilder,
  UpdateBuilder,
  RenameBuilder,
  DeleteBuilder,

  // alerter
  CreateAlerter,
  UpdateAlerter,
  RenameAlerter,
  DeleteAlerter,
  TestAlerter,

  // sync
  CreateResourceSync,
  UpdateResourceSync,
  RenameResourceSync,
  DeleteResourceSync,
  WriteSyncContents,
  CommitSync,
  RunSync,

  // variable
  CreateVariable,
  UpdateVariableValue,
  DeleteVariable,

  // git provider
  CreateGitProviderAccount,
  UpdateGitProviderAccount,
  DeleteGitProviderAccount,

  // docker registry
  CreateDockerRegistryAccount,
  UpdateDockerRegistryAccount,
  DeleteDockerRegistryAccount,
}

#[typeshare]
#[derive(
  Serialize,
  Deserialize,
  Debug,
  Default,
  Display,
  EnumString,
  PartialEq,
  Hash,
  Eq,
  Clone,
  Copy,
)]
pub enum SearchCombinator {
  #[default]
  Or,
  And,
}

#[typeshare]
#[derive(
  Serialize,
  Deserialize,
  Debug,
  PartialEq,
  Hash,
  Eq,
  Clone,
  Copy,
  Default,
  Display,
  EnumString,
)]
#[serde(rename_all = "UPPERCASE")]
#[strum(serialize_all = "UPPERCASE")]
pub enum TerminationSignal {
  #[serde(alias = "1")]
  SigHup,
  #[serde(alias = "2")]
  SigInt,
  #[serde(alias = "3")]
  SigQuit,
  #[default]
  #[serde(alias = "15")]
  SigTerm,
}

/// Used to reference a specific resource across all resource types
#[typeshare]
#[derive(
  Debug,
  Clone,
  PartialEq,
  Eq,
  Hash,
  Serialize,
  Deserialize,
  EnumVariants,
)]
#[variant_derive(
  Debug,
  Clone,
  Copy,
  PartialEq,
  Eq,
  PartialOrd,
  Ord,
  Hash,
  Serialize,
  Deserialize,
  Display,
  EnumString,
  AsRefStr
)]
#[serde(tag = "type", content = "id")]
pub enum ResourceTarget {
  System(String),
  Server(String),
  Stack(String),
  Deployment(String),
  Build(String),
  Repo(String),
  Procedure(String),
  Action(String),
  Builder(String),
  Alerter(String),
  ResourceSync(String),
}

impl ResourceTarget {
  pub fn extract_variant_id(
    &self,
  ) -> (ResourceTargetVariant, &String) {
    let id = match &self {
      ResourceTarget::System(id) => id,
      ResourceTarget::Server(id) => id,
      ResourceTarget::Stack(id) => id,
      ResourceTarget::Build(id) => id,
      ResourceTarget::Builder(id) => id,
      ResourceTarget::Deployment(id) => id,
      ResourceTarget::Repo(id) => id,
      ResourceTarget::Alerter(id) => id,
      ResourceTarget::Procedure(id) => id,
      ResourceTarget::Action(id) => id,
      ResourceTarget::ResourceSync(id) => id,
    };
    (self.extract_variant(), id)
  }

  pub fn system() -> ResourceTarget {
    Self::System("system".to_string())
  }
}

impl Default for ResourceTarget {
  fn default() -> Self {
    ResourceTarget::system()
  }
}

impl From<&build::Build> for ResourceTarget {
  fn from(build: &build::Build) -> Self {
    Self::Build(build.id.clone())
  }
}

impl From<&deployment::Deployment> for ResourceTarget {
  fn from(deployment: &deployment::Deployment) -> Self {
    Self::Deployment(deployment.id.clone())
  }
}

impl From<&server::Server> for ResourceTarget {
  fn from(server: &server::Server) -> Self {
    Self::Server(server.id.clone())
  }
}

impl From<&repo::Repo> for ResourceTarget {
  fn from(repo: &repo::Repo) -> Self {
    Self::Repo(repo.id.clone())
  }
}

impl From<&builder::Builder> for ResourceTarget {
  fn from(builder: &builder::Builder) -> Self {
    Self::Builder(builder.id.clone())
  }
}

impl From<&alerter::Alerter> for ResourceTarget {
  fn from(alerter: &alerter::Alerter) -> Self {
    Self::Alerter(alerter.id.clone())
  }
}

impl From<&procedure::Procedure> for ResourceTarget {
  fn from(procedure: &procedure::Procedure) -> Self {
    Self::Procedure(procedure.id.clone())
  }
}

impl From<&sync::ResourceSync> for ResourceTarget {
  fn from(resource_sync: &sync::ResourceSync) -> Self {
    Self::ResourceSync(resource_sync.id.clone())
  }
}

impl From<&stack::Stack> for ResourceTarget {
  fn from(stack: &stack::Stack) -> Self {
    Self::Stack(stack.id.clone())
  }
}

impl From<&action::Action> for ResourceTarget {
  fn from(action: &action::Action) -> Self {
    Self::Action(action.id.clone())
  }
}

impl ResourceTargetVariant {
  /// These need to use snake case
  pub fn toml_header(&self) -> &'static str {
    match self {
      ResourceTargetVariant::System => "system",
      ResourceTargetVariant::Build => "build",
      ResourceTargetVariant::Builder => "builder",
      ResourceTargetVariant::Deployment => "deployment",
      ResourceTargetVariant::Server => "server",
      ResourceTargetVariant::Repo => "repo",
      ResourceTargetVariant::Alerter => "alerter",
      ResourceTargetVariant::Procedure => "procedure",
      ResourceTargetVariant::ResourceSync => "resource_sync",
      ResourceTargetVariant::Stack => "stack",
      ResourceTargetVariant::Action => "action",
    }
  }
}

#[typeshare]
#[derive(
  Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize,
)]
pub enum ScheduleFormat {
  #[default]
  English,
  Cron,
}

/// Used with ExecuteTerminal to capture the exit code
pub const KOMODO_EXIT_CODE: &str = "__KOMODO_EXIT_CODE:";
