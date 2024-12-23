use tracing::subscriber::set_global_default;
use tracing_subscriber::{
    fmt::{format::FmtSpan, time},
    EnvFilter,
    layer::SubscriberExt,
    Registry,
};

pub fn init_logging(service_name: &str) {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    let formatting_layer = tracing_subscriber::fmt::layer()
        .with_timer(time::time())  
        .with_target(true)
        .with_thread_ids(true)
        .with_span_events(FmtSpan::CLOSE)
        .with_file(true)
        .with_line_number(true);

    let subscriber = Registry::default()
        .with(env_filter)
        .with(formatting_layer);

    set_global_default(subscriber).expect("Failed to set tracing subscriber");
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracing::info;

    #[test]
    fn test_logging_initialization() {
        assert!(init_logging().is_ok());
        
        info!("Test log message");
    }
}
