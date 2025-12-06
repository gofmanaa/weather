mod commands;
mod config;
mod errors;

mod provider_registry;

mod app;
mod logger;
mod weather_providers;

use crate::app::WeatherApp;
use crate::commands::{default_settings_path, run};
use crate::config::init_settings_file;
use crate::logger::init_logger;
use crate::provider_registry::build_registry;
use crate::{config::load_settings, errors::AppError};
use clap::Parser;
use commands::Cli;
use tracing::{info, trace};

#[tokio::main]
async fn main() -> Result<(), AppError> {
    // add ratatui TUI
    let _logger_guard = init_logger();
    let _ = dotenvy::dotenv().ok();
    info!("App started");
    let cli = Cli::parse();

    let _ = init_settings_file(&default_settings_path());

    let settings = load_settings(cli.config_path.clone().as_path()).map_err(AppError::Config)?;

    trace!("Settings {:?}", settings);

    let registry = build_registry(&settings)?;
    let app = WeatherApp::new(registry);

    run(cli, app, settings).await
}
