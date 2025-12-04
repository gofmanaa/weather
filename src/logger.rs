use dotenvy::var;
use std::io;
use tracing::{info, trace};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};

#[derive(Debug)]
pub struct LoggerGuard {
    _std_out_guard: WorkerGuard,
}
pub fn init_logger() -> LoggerGuard {
    let (std_out_writer, std_out_guard) = tracing_appender::non_blocking(io::stdout());

    let enable_color = var("ENABLE_COLOR").map(|v| v == "true").unwrap_or(false);

    let std_out_layer = fmt::layer()
        .with_writer(std_out_writer)
        .with_ansi(enable_color)
        .with_target(false)
        .with_level(true)
        .without_time()
        .with_filter(EnvFilter::try_from_default_env().unwrap_or(EnvFilter::new("debug")));

    tracing_subscriber::registry().with(std_out_layer).init();

    trace!("Logging successfully initialized!");
    info!("Enabling ANSI: {}", enable_color);

    LoggerGuard {
        _std_out_guard: std_out_guard,
    }
}
