use std::time::Duration;

use opentelemetry::{global, trace::TracerProvider, KeyValue};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{
  trace::{Sampler, Tracer},
  Resource,
};
use opentelemetry_semantic_conventions::{
  resource::{SERVICE_NAME, SERVICE_VERSION},
  SCHEMA_URL,
};

fn resource(service_name: String) -> Resource {
  Resource::from_schema_url(
    [
      KeyValue::new(SERVICE_NAME, service_name),
      KeyValue::new(SERVICE_VERSION, env!("CARGO_PKG_VERSION")),
    ],
    SCHEMA_URL,
  )
}

pub fn tracer(endpoint: &str, service_name: String) -> Tracer {
  let provider = opentelemetry_sdk::trace::TracerProvider::builder()
    .with_resource(resource(service_name.clone()))
    .with_sampler(Sampler::AlwaysOn)
    .with_batch_exporter(
      opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(endpoint)
        .with_timeout(Duration::from_secs(3))
        .build()
        .unwrap(),
      opentelemetry_sdk::runtime::Tokio,
    )
    .build();
  global::set_tracer_provider(provider.clone());
  provider.tracer(service_name)
}
