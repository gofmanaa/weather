use async_trait::async_trait;
use chrono::NaiveDate;

pub mod error;
pub mod openweather;
pub mod weatherapi;

use crate::weather_providers::error::ProviderError;

#[derive(Debug)]
pub struct WeatherData {
    pub location: String,
    pub timestamp: String,
    pub temp_c: f64,
    pub condition: String,
    pub wind_kph: Option<f64>,
}

#[async_trait]
pub trait WeatherProvider: Send + Sync {
    async fn fetch(
        &self,
        location: &str,
        date: Option<NaiveDate>,
    ) -> Result<WeatherData, ProviderError>;
}
