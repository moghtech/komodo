use std::time::Duration;

use opentelemetry::{KeyValue, global, trace::TracerProvider};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{
  Resource,
  trace::{Sampler, Tracer},
};
use opentelemetry_semantic_conventions::resource::SERVICE_VERSION;

fn resource(service_name: String) -> Resource {
  Resource::builder()
    .with_service_name(service_name)
    .with_attribute(KeyValue::new(
      SERVICE_VERSION,
      env!("CARGO_PKG_VERSION"),
    ))
    .build()
}

pub fn tracer(endpoint: &str, service_name: String) -> Tracer {
  let provider =
    opentelemetry_sdk::trace::TracerProviderBuilder::default()
      .with_resource(resource(service_name.clone()))
      .with_sampler(Sampler::AlwaysOn)
      .with_batch_exporter(
        opentelemetry_otlp::SpanExporter::builder()
          .with_http()
          .with_endpoint(endpoint)
          .with_timeout(Duration::from_secs(3))
          .build()
          .unwrap(),
      )
      .build();
  global::set_tracer_provider(provider.clone());
  provider.tracer(service_name)
}
