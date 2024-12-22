use tracing::{subscriber::set_global_default, Subscriber};
use tracing_subscriber::{
    fmt::{format::FmtSpan, time::ChronoUtc},
    layer::SubscriberExt,
    EnvFilter, Registry,
};

pub fn init_logging() -> Result<(), Box<dyn std::error::Error>> {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    let formatting_layer = tracing_subscriber::fmt::layer()
        .with_timer(ChronoUtc::rfc3339())
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_target(true)
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .with_file(true)
        .with_line_number(true)
        .json();

    let subscriber = Registry::default()
        .with(env_filter)
        .with(formatting_layer);

    set_global_default(subscriber)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracing::info;

    #[test]
    fn test_logging_initialization() {
        assert!(init_logging().is_ok());
        
        // Test logging
        info!("Test log message");
    }
}
