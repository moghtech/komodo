use anyhow::Context;
use komodo_client::entities::logger::{LogConfig, StdioLogMode};
use tracing::level_filters::LevelFilter;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::{
  Registry, layer::SubscriberExt, util::SubscriberInitExt,
};

mod otel;

pub fn init(config: &LogConfig) -> anyhow::Result<()> {
  let log_level: tracing::Level = config.level.into();

  let registry =
    Registry::default().with(LevelFilter::from(log_level));

  let use_otel = !config.otlp_endpoint.is_empty();

  match (config.stdio, use_otel, config.pretty) {
    (StdioLogMode::Standard, true, true) => {
      let tracer = otel::tracer(
        &config.otlp_endpoint,
        config.opentelemetry_service_name.clone(),
      );
      registry
        .with(tracing_subscriber::fmt::layer().pretty())
        .with(OpenTelemetryLayer::new(tracer))
        .try_init()
    }
    (StdioLogMode::Standard, true, false) => {
      let tracer = otel::tracer(
        &config.otlp_endpoint,
        config.opentelemetry_service_name.clone(),
      );
      registry
        .with(tracing_subscriber::fmt::layer())
        .with(OpenTelemetryLayer::new(tracer))
        .try_init()
    }

    (StdioLogMode::Json, true, true) => {
      let tracer = otel::tracer(
        &config.otlp_endpoint,
        config.opentelemetry_service_name.clone(),
      );
      registry
        .with(tracing_subscriber::fmt::layer().json().pretty())
        .with(OpenTelemetryLayer::new(tracer))
        .try_init()
    }
    (StdioLogMode::Json, true, false) => {
      let tracer = otel::tracer(
        &config.otlp_endpoint,
        config.opentelemetry_service_name.clone(),
      );
      registry
        .with(tracing_subscriber::fmt::layer().json())
        .with(OpenTelemetryLayer::new(tracer))
        .try_init()
    }

    (StdioLogMode::Standard, false, true) => registry
      .with(tracing_subscriber::fmt::layer().pretty())
      .try_init(),
    (StdioLogMode::Standard, false, false) => {
      registry.with(tracing_subscriber::fmt::layer()).try_init()
    }

    (StdioLogMode::Json, false, true) => registry
      .with(tracing_subscriber::fmt::layer().json().pretty())
      .try_init(),
    (StdioLogMode::Json, false, false) => registry
      .with(tracing_subscriber::fmt::layer().json())
      .try_init(),

    (StdioLogMode::None, true, _) => {
      let tracer = otel::tracer(
        &config.otlp_endpoint,
        config.opentelemetry_service_name.clone(),
      );
      registry.with(OpenTelemetryLayer::new(tracer)).try_init()
    }
    (StdioLogMode::None, false, _) => Ok(()),
  }
  .context("failed to init logger")
}
