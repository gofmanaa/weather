mod commands;
mod config;
mod errors;

mod provider_manager;
mod provider_registry;

mod app;
mod weather_providers;

use crate::app::WeatherApp;
use crate::commands::run;
use crate::provider_manager::ProviderManager;
use crate::provider_registry::build_registry;
use crate::{config::load_settings, errors::AppError};
use clap::Parser;
use commands::Cli;

#[tokio::main]
async fn main() -> Result<(), AppError> {
    let cli = Cli::parse();
    println!("{:?}", cli);

    let settings = load_settings(cli.config_path.clone().unwrap_or_default().as_path())
        .map_err(|e| AppError::Config(e))?;

    let registry = build_registry();
    let manager = ProviderManager::new(registry);
    let app = WeatherApp::new(manager);

    run(cli, app, settings).await
}
