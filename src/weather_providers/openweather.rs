use crate::weather_providers::error::ProviderError;
use crate::weather_providers::{WeatherData, WeatherProvider};
use chrono::{DateTime, Local, NaiveDate};
use openweathermap::CurrentWeather;
use tracing::debug;

impl From<CurrentWeather> for WeatherData {
    fn from(w: CurrentWeather) -> Self {
        let dt = DateTime::from_timestamp(w.dt, 0)
            .map_or_else(Local::now, |utc| utc.with_timezone(&Local));

        WeatherData {
            location: w.name,
            datetime: dt.to_rfc3339(),
            temp_c: w.main.temp,
            humidity: w.main.humidity,
            pressure: w.main.pressure,
            condition: w
                .weather
                .first()
                .map_or_else(|| "unknown".to_string(), |c| c.description.clone()),
            wind_kph: w.wind.speed * 3.6,
            wind_deg: w.wind.deg,
        }
    }
}

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

        Ok(WeatherData::from(weather_response))
    }
}
