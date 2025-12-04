use crate::weather_providers::error::ProviderError;
use crate::weather_providers::{WeatherData, WeatherProvider};
use chrono::{DateTime, Local, NaiveDate};
use openweathermap::CurrentWeather;
use tracing::debug;

pub struct OpenWeather {
    api_key: String,
}

impl OpenWeather {
    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }

    pub fn get_weather(&self, location: &str) -> Result<CurrentWeather, String> {
        debug!("Api key: {}", self.api_key);
        openweathermap::blocking::weather(location, "metric", "en", &self.api_key)
    }
}

#[async_trait::async_trait]
impl WeatherProvider for OpenWeather {
    async fn fetch(
        &self,
        location: &str,
        _date: Option<NaiveDate>,
    ) -> Result<WeatherData, ProviderError> {
        let weather_response = self
            .get_weather(location)
            .map_err(ProviderError::ApiRequest)?;

        let local_dt: DateTime<Local> = match DateTime::from_timestamp(weather_response.dt, 0) {
            Some(utc_dt) => utc_dt.with_timezone(&Local),
            None => Local::now(),
        };

        Ok(WeatherData {
            location: weather_response.name,
            datetime: local_dt.to_string(),
            temp_c: weather_response.main.temp,
            humidity: weather_response.main.humidity,
            pressure: weather_response.main.pressure,
            condition: weather_response.weather[0].description.clone(),
            wind_kph: weather_response.wind.speed,
            wind_deg: weather_response.wind.deg,
        })
    }
}
