use async_trait::async_trait;
use chrono::{DateTime, Local, NaiveDateTime, Utc};
use std::fmt::{Display, Formatter};

pub mod error;
pub mod openweather;
pub mod weatherapi;

use crate::weather_providers::error::ProviderError;

/// Represents the weather information for a specific location.
#[derive(Debug, Default)]
pub struct WeatherData {
    /// The name of the city or location.
    pub location: String,
    /// The local date and time when this weather data was recorded, in human-readable format.
    pub datetime: DateTime<Utc>,
    /// Temperature in Celsius.
    pub temp_c: f64,
    /// Humidity percentage (0–100%).
    pub humidity: f64,
    /// Atmospheric pressure in hPa (hectopascals).
    pub pressure: f64,
    /// A short textual description of the weather condition (e.g., "Sunny", "Cloudy").
    pub condition: String,
    /// Wind speed in kilometers per hour.
    pub wind_kph: f64,
    /// Wind direction in degrees (meteorological standard, 0–360°).
    pub wind_deg: f64,
}

impl Display for WeatherData {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Weather in {}: {} {}\n> Date: {}\n> Temperature: {:.1}°C\n> Humidity: {:.1}%\n> Pressure: {:.1} hPa\n> Wind: {:.1} km/h at {:.1}°",
            self.location,
            self.condition,
            get_temperature_emoji(self.temp_c),
            self.datetime.with_timezone(&Local),
            self.temp_c,
            self.humidity,
            self.pressure,
            self.wind_kph,
            self.wind_deg
        )
    }
}

fn get_temperature_emoji(temperature: f64) -> &'static str {
    match temperature {
        t if t < 0.0 => "❄️",
        t if (0.0..10.0).contains(&t) => "☁️",
        t if (10.0..20.0).contains(&t) => "⛅",
        t if (20.0..30.0).contains(&t) => "🌤️",
        _ => "🔥",
    }
}

#[async_trait]
pub trait WeatherProvider: Send + Sync {
    async fn fetch(
        &self,
        location: &str,
        date: Option<NaiveDateTime>,
    ) -> Result<WeatherData, ProviderError>;
}
