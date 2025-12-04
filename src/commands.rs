use crate::app::WeatherApp;
use crate::config::save_settings;
use crate::errors::AppError;
use crate::provider_manager::ProviderKind;
use crate::provider_registry::ProviderRegistry;
use chrono::{DateTime, Local, NaiveDate, NaiveDateTime};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(author, version, about)]
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

    Err(AppError::InvalidProvider(format!(
        "Invalid datetime format: {}",
        s
    )))
}

pub async fn run(
    cli: Cli,
    wapp: WeatherApp<ProviderRegistry>,
    mut settings: crate::config::Settings,
) -> Result<(), AppError> {
    let config_path = cli.config_path.unwrap_or_default();
    if let Some(command) = cli.command {
        match command {
            Commands::Configure { provider } => {
                if let Some(provider) = provider {
                    if let Ok(p) = ProviderKind::from_str(provider.as_str()) {
                        settings.default_provider = p.name().to_string();
                        save_settings(&settings, config_path).map_err(|e| AppError::Config(e))?;
                        println!("Settings {:?}", settings);
                    } else {
                        eprintln!("Provider `{}` not supported", provider);
                    }
                } else {
                    println!("Settings default_provider: {:?}", settings.default_provider);
                }
            }
            Commands::Get { address, date } => {
                println!("Cli {:?}", address);
                println!("Cli {:?}", date);
                println!("Settings {:?}", settings);

                let res = wapp.run(&settings.default_provider, &address, date).await?;
                println!("{:#?}", res);
            }
        }
    } else {
        println!("Settings {:?}", settings);
    }
    Ok(())
}
