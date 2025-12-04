use crate::weather_providers::error::ProviderError;
use crate::weather_providers::{WeatherData, WeatherProvider};
use chrono::{DateTime, NaiveDate, TimeZone, Utc};
use serde::Deserialize;

const OPEN_WEATHER_API_URL: &str = "https://api.openweathermap.org/";
#[derive(Debug, Deserialize)]
struct WeatherResponse {
    name: String,
    dt: i64,
    main: Main,
    weather: Vec<WeatherCondition>,
    wind: Wind,
}

#[derive(Debug, Deserialize)]
struct Main {
    temp: f64,
}

#[derive(Debug, Deserialize)]
struct WeatherCondition {
    description: String,
}

#[derive(Debug, Deserialize)]
struct Wind {
    speed: f64,
}

pub struct OpenWeather {
    api_key: String,
    base_url: String,
}

impl OpenWeather {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            base_url: OPEN_WEATHER_API_URL.to_string(),
        }
    }

    pub fn with_base_url(mut self, base_url: String) -> Self {
        self.base_url = base_url;
        self
    }
}

#[async_trait::async_trait]
impl WeatherProvider for OpenWeather {
    async fn fetch(
        &self,
        location: &str,
        _date: Option<NaiveDate>,
    ) -> Result<WeatherData, ProviderError> {
        let url = format!(
            "{}/data/2.5/weather?q={}&appid={}&units=metric",
            self.base_url, location, self.api_key
        );

        let res = reqwest::get(&url).await?;
        let weather_response: WeatherResponse = res.json().await?;

        Ok(WeatherData {
            location: weather_response.name,
            timestamp: Utc
                .from_utc_datetime(
                    &DateTime::from_timestamp(weather_response.dt, 0)
                        .unwrap()
                        .naive_utc(),
                )
                .to_string(),
            temp_c: weather_response.main.temp,
            condition: weather_response.weather[0].description.clone(),
            wind_kph: Some(weather_response.wind.speed * 3.6), // m/s to km/h
        })
    }
}
