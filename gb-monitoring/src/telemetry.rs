use opentelemetry::{
    sdk::{trace, Resource},
    runtime::Tokio,
    KeyValue,
};
use opentelemetry_otlp::{Protocol, WithExportConfig};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TelemetryError {
    #[error("Failed to initialize tracer: {0}")]
    Init(String),
}

#[allow(dead_code)]
pub struct Telemetry {
    tracer: trace::Tracer    
}

impl Telemetry {
    pub async fn new(service_name: &str) -> Result<Self, TelemetryError> {
        let tracer = Self::init_tracer(service_name)
            .await
            .map_err(|e| TelemetryError::Init(e.to_string()))?;
        Ok(Self { tracer })
    }

    async fn init_tracer(service_name: &str) -> Result<trace::Tracer, TelemetryError> {
        let exporter = opentelemetry_otlp::new_exporter()
            .tonic()
            .with_protocol(Protocol::Grpc);
            
        let resource = Resource::new(vec![
            KeyValue::new("service.name", service_name.to_string()),
        ]);
            
        let config = trace::config().with_resource(resource);
            
        let tracer = opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_exporter(exporter)
            .with_trace_config(config)
            .install_batch(Tokio)
            .map_err(|e| TelemetryError::Init(e.to_string()))?;

        Ok(tracer)
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

    #[tokio::test]
    async fn test_telemetry_creation() {
        let telemetry = Telemetry::new("test-service").await;
        assert!(telemetry.is_ok());
    }
}