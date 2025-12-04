use async_trait::async_trait;
use chrono::NaiveDate;

pub mod error;
pub mod openweather;
pub mod weatherapi;

use crate::weather_providers::error::ProviderError;

/// Represents the weather information for a specific location.
///
/// # Fields
///
/// - `location`: The name of the city or location.
/// - `datetime`: The local date and time when this weather data was recorded, in human-readable format.
/// - `temp_c`: Temperature in Celsius.
/// - `humidity`: Humidity percentage (0–100%).
/// - `pressure`: Atmospheric pressure in hPa (hectopascals).
/// - `condition`: A short textual description of the weather condition (e.g., "Sunny", "Cloudy").
/// - `wind_kph`: Wind speed in kilometers per hour.
/// - `wind_deg`: Wind direction in degrees (meteorological standard, 0–360°).
#[derive(Debug)]
pub struct WeatherData {
    pub location: String,
    pub datetime: String,
    pub temp_c: f64,
    pub humidity: f64,
    pub pressure: f64,
    pub condition: String,
    pub wind_kph: f64,
    pub wind_deg: f64,
}

#[async_trait]
pub trait WeatherProvider: Send + Sync {
    async fn fetch(
        &self,
        location: &str,
        date: Option<NaiveDate>,
    ) -> Result<WeatherData, ProviderError>;
}
