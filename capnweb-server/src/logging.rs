use std::path::Path;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{
    fmt::{self, time::ChronoUtc},
    prelude::*,
    EnvFilter,
};

/// Initialize logging with file output and console output
pub fn init_logging(log_dir: impl AsRef<Path>, log_prefix: &str) -> anyhow::Result<()> {
    let log_dir_path = log_dir.as_ref();

    // Create log directory if it doesn't exist
    std::fs::create_dir_all(log_dir_path)?;

    // Create rolling file appender (daily rotation)
    let file_appender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix(log_prefix)
        .build(log_dir_path)?;

    // Create non-blocking writer for file output
    let (file_writer, _guard) = tracing_appender::non_blocking(file_appender);

    // Set up environment filter with sensible defaults
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        // Default to info level for our crates, warn for others
        EnvFilter::new("capnweb=debug,tower_http=debug,axum=debug,hyper=debug,warn")
    });

    // Create console subscriber
    let console_layer = fmt::layer()
        .with_target(true)
        .with_thread_ids(true)
        .with_timer(ChronoUtc::rfc_3339())
        .with_writer(std::io::stderr);

    // Create file subscriber
    let file_layer = fmt::layer()
        .with_target(true)
        .with_thread_ids(true)
        .with_timer(ChronoUtc::rfc_3339())
        .with_ansi(false) // No color codes in files
        .with_writer(file_writer);

    // Combine layers
    tracing_subscriber::registry()
        .with(env_filter)
        .with(console_layer)
        .with(file_layer)
        .init();

    // Keep the guard alive by leaking it
    std::mem::forget(_guard);

    tracing::info!("Logging initialized with file output to {:?}", log_dir_path);
    Ok(())
}

/// Initialize simple console-only logging for tests
pub fn init_test_logging() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("capnweb=trace,debug")),
        )
        .try_init();
}
