use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LogConfig {
  /// The logging level. default: info
  #[serde(default)]
  pub level: LogLevel,

  /// Controls logging to stdout / stderr
  #[serde(default)]
  pub stdio: StdioLogMode,

  /// Use tracing-subscriber's pretty logging output option.
  #[serde(default)]
  pub pretty: bool,

  /// Including information about the log location (ie the function which produced the log).
  /// Tracing refers to this as the 'target'.
  #[serde(default = "default_location")]
  pub location: bool,

  /// Logs use ansi colors for readability.
  #[serde(default = "default_ansi")]
  pub ansi: bool,

  /// Enable opentelemetry exporting
  #[serde(default)]
  pub otlp_endpoint: String,

  #[serde(default = "default_opentelemetry_service_name")]
  pub opentelemetry_service_name: String,

  #[serde(default = "default_opentelemetry_scope_name")]
  pub opentelemetry_scope_name: String,
}

fn default_opentelemetry_service_name() -> String {
  String::from("Komodo")
}

fn default_opentelemetry_scope_name() -> String {
  String::from("Komodo")
}

fn default_location() -> bool {
  false
}

fn default_ansi() -> bool {
  true
}

impl Default for LogConfig {
  fn default() -> Self {
    Self {
      level: Default::default(),
      stdio: Default::default(),
      pretty: Default::default(),
      location: default_location(),
      ansi: default_ansi(),
      otlp_endpoint: Default::default(),
      opentelemetry_service_name: default_opentelemetry_service_name(
      ),
      opentelemetry_scope_name: default_opentelemetry_scope_name(),
    }
  }
}

fn default_log_config() -> &'static LogConfig {
  static DEFAULT_LOG_CONFIG: OnceLock<LogConfig> = OnceLock::new();
  DEFAULT_LOG_CONFIG.get_or_init(Default::default)
}

impl LogConfig {
  pub fn is_default(&self) -> bool {
    self == default_log_config()
  }
}

#[derive(
  Debug,
  Clone,
  Copy,
  Default,
  PartialEq,
  Eq,
  Hash,
  Serialize,
  Deserialize,
)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
  Trace,
  Debug,
  #[default]
  Info,
  Warn,
  Error,
}

impl From<LogLevel> for tracing::Level {
  fn from(value: LogLevel) -> Self {
    match value {
      LogLevel::Trace => tracing::Level::TRACE,
      LogLevel::Debug => tracing::Level::DEBUG,
      LogLevel::Info => tracing::Level::INFO,
      LogLevel::Warn => tracing::Level::WARN,
      LogLevel::Error => tracing::Level::ERROR,
    }
  }
}

impl From<tracing::Level> for LogLevel {
  fn from(value: tracing::Level) -> Self {
    match value.as_str() {
      "trace" => LogLevel::Trace,
      "debug" => LogLevel::Debug,
      "info" => LogLevel::Info,
      "warn" => LogLevel::Warn,
      "error" => LogLevel::Error,
      _ => LogLevel::Info,
    }
  }
}

#[derive(
  Debug,
  Clone,
  Copy,
  Default,
  PartialEq,
  Eq,
  Hash,
  Serialize,
  Deserialize,
)]
#[serde(rename_all = "lowercase")]
pub enum StdioLogMode {
  #[default]
  Standard,
  Json,
  None,
}

#[cfg(feature = "logger")]
impl mogh_logger::LogConfig for &LogConfig {
  fn ansi(&self) -> bool {
    self.ansi
  }
  fn level(&self) -> tracing::Level {
    self.level.into()
  }
  fn location(&self) -> bool {
    self.location
  }
  fn opentelemetry_scope_name(&self) -> String {
    self.opentelemetry_scope_name.clone()
  }
  fn opentelemetry_service_name(&self) -> String {
    self.opentelemetry_service_name.clone()
  }
  fn otlp_endpoint(&self) -> &str {
    &self.otlp_endpoint
  }
  fn pretty(&self) -> bool {
    self.pretty
  }
  fn stdio(&self) -> mogh_logger::StdioLogMode {
    match self.stdio {
      StdioLogMode::Standard => mogh_logger::StdioLogMode::Standard,
      StdioLogMode::Json => mogh_logger::StdioLogMode::Json,
      StdioLogMode::None => mogh_logger::StdioLogMode::None,
    }
  }
}
