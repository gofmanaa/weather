use crate::app::WeatherApp;
use crate::config::save_settings;
use crate::errors::AppError;
use crate::provider_registry::ProviderRegistry;
use crate::weather_providers::WeatherData;
use chrono::{DateTime, Local, NaiveDate, NaiveDateTime};
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing::{debug, info};

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
    wapp: WeatherApp<ProviderRegistry>,
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
    let description = &response.condition;
    let datetime = &response.datetime;
    let temperature = response.temp_c;
    let humidity = response.humidity;
    let pressure = response.pressure;
    let wind_speed = response.wind_kph;
    let wind_deg = response.wind_deg;

    let weather_text = format!(
        "Weather in {}: {} {}
> DateTeme: {},
> Temperature: {:.1}°C,
> Humidity: {:.1} %,
> Pressure: {:.1} hPa,
> Wind Speed: {:.1} k/h
> Wind Degree: {:.1}°
Provider: {}",
        response.location,
        description,
        get_temperature_emoji(temperature),
        datetime,
        temperature,
        humidity,
        pressure,
        wind_speed,
        wind_deg,
        provider.to_uppercase(),
    );

    println!("{weather_text}");
}

fn get_temperature_emoji(temperature: f64) -> &'static str {
    if temperature < 0.0 {
        "❄️"
    } else if (0.0..10.0).contains(&temperature) {
        "☁️"
    } else if (10.0..20.0).contains(&temperature) {
        "⛅"
    } else if (20.0..30.0).contains(&temperature) {
        "🌤️"
    } else {
        "🔥"
    }
}
