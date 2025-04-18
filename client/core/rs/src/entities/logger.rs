use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogConfig {
  /// The logging level. default: info
  #[serde(default)]
  pub level: LogLevel,

  /// Controls logging to stdout / stderr
  #[serde(default)]
  pub stdio: StdioLogMode,

  #[serde(default)]
  pub pretty: bool,

  /// Enable opentelemetry exporting
  #[serde(default)]
  pub otlp_endpoint: String,

  #[serde(default = "default_opentelemetry_service_name")]
  pub opentelemetry_service_name: String,
}

fn default_opentelemetry_service_name() -> String {
  String::from("Komodo")
}

impl Default for LogConfig {
  fn default() -> Self {
    Self {
      level: Default::default(),
      stdio: Default::default(),
      pretty: Default::default(),
      otlp_endpoint: Default::default(),
      opentelemetry_service_name: default_opentelemetry_service_name(
      ),
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
