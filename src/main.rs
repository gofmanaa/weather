mod commands;
mod config;
mod errors;

mod provider_manager;
mod provider_registry;

mod app;
mod logger;
mod weather_providers;

use crate::app::WeatherApp;
use crate::commands::run;
use crate::logger::init_logger;
use crate::provider_manager::ProviderManager;
use crate::provider_registry::build_registry;
use crate::{config::load_settings, errors::AppError};
use clap::Parser;
use commands::Cli;
use tracing::{info, trace};

#[tokio::main]
async fn main() -> Result<(), AppError> {
    let _logger_guard = init_logger();
    let _ = dotenvy::dotenv().ok();
    info!("App started");
    let cli = Cli::parse();

    let settings = load_settings(cli.config_path.clone().unwrap_or_default().as_path())
        .map_err(AppError::Config)?;

    trace!("Settings {:?}", settings);

    let registry = build_registry()?;
    let manager = ProviderManager::new(registry);
    let app = WeatherApp::new(manager);

    run(cli, app, settings).await
}
