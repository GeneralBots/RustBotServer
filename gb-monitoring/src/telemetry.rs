use opentelemetry::{
    runtime::Tokio,
    sdk::{trace, Resource},
    KeyValue,
};
use std::time::Duration;
use tracing::error;

pub struct Telemetry {
    tracer: opentelemetry::sdk::trace::Tracer,
}

impl Telemetry {
    pub fn new(service_name: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let tracer = opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_exporter(
                opentelemetry_otlp::new_exporter()
                    .tonic()
                    .with_endpoint("http://localhost:4317")
                    .with_timeout(Duration::from_secs(3))
            )
            .with_trace_config(
                trace::config()
                    .with_resource(Resource::new(vec![KeyValue::new(
                        "service.name",
                        service_name.to_string(),
                    )]))
                    .with_sampler(trace::Sampler::AlwaysOn)
            )
            .install_batch(Tokio)?;

        Ok(Self { tracer })
    }

    pub fn tracer(&self) -> &opentelemetry::sdk::trace::Tracer {
        &self.tracer
    }
}

impl Drop for Telemetry {
    fn drop(&mut self) {
        opentelemetry::global::shutdown_tracer_provider();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_telemetry_creation() {
        let telemetry = Telemetry::new("test-service");
        assert!(telemetry.is_ok());
    }
}
