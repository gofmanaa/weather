use crate::app::WeatherApp;
use crate::config::save_settings;
use crate::errors::AppError;
use crate::weather_providers::WeatherData;
use chrono::{DateTime, Local, NaiveDate, NaiveDateTime};
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing::{debug, info, warn};

#[derive(Debug, Parser)]
#[command(author, version, about, arg_required_else_help = true)]
pub struct Cli {
    #[command(subcommand)]
    pub(crate) command: Option<Commands>,

    #[arg(short, long, value_name = "CONF_FILE", default_value = "settings.toml")]
    pub(crate) config_path: Option<PathBuf>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Configure {
        provider: Option<String>,
    },
    Get {
        address: String,
        #[arg(long, value_parser = parse_datetime)]
        date: Option<NaiveDate>,
    },
}

fn parse_datetime(s: &str) -> Result<NaiveDate, AppError> {
    // RFC3339 format
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Ok(dt.with_timezone(&Local).date_naive());
    }

    //  "YYYY-MM-DD HH:MM:SS"
    if let Ok(ndt) = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S") {
        return Ok(ndt.date());
    }

    //  "YYYY-MM-DD"
    if let Ok(date) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        return Ok(date);
    }

    Err(AppError::InvalidDate(s.to_string()))
}

pub async fn run(
    cli: Cli,
    wapp: WeatherApp,
    mut settings: crate::config::Settings,
) -> Result<(), AppError> {
    let config_path = cli.config_path.unwrap_or_default();

    if let Some(command) = cli.command {
        match command {
            Commands::Configure { provider } => {
                if let Some(provider) = provider {
                    info!("User provider: {}", provider);
                    let provider = provider.to_lowercase();
                    if wapp.provider_exist(&provider) {
                        settings.default_provider = provider;
                        save_settings(&settings, &config_path).map_err(AppError::Config)?;
                        println!("Default provider saved to {}", config_path.display());
                    } else {
                        warn!("Provider `{provider}` not supported");
                        eprintln!("Provider `{provider}` not supported");
                    }
                } else {
                    println!("Default provider: {}", settings.default_provider);
                    println!("Available providers: {:?}", wapp.list());
                }
            },
            Commands::Get { address, date } => {
                debug!("Cli address: {}", address);
                debug!("Cli date: {:?}", date);
                debug!("Provider: {:?}", settings.default_provider);

                let res = wapp.run(&settings.default_provider, &address, date).await?;
                debug!("{:#?}", res);

                display_weather_info(&res, &settings.default_provider);
            },
        }
    }

    Ok(())
}

fn display_weather_info(response: &WeatherData, provider: &str) {
    println!("{}\nProvider: {}", response, provider.to_uppercase());
}
