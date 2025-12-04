use crate::weather_providers::error::ProviderError;
use crate::weather_providers::{WeatherData, WeatherProvider};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherResponse {
    pub current: WeatherCondition,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherCondition {
    pub last_updated: String,
    pub temp_c: f64,
    pub temp_f: f64,
    pub condition: ConditionFields,
    #[serde(flatten)]
    #[allow(dead_code)]
    extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionFields {
    pub text: String,
    pub icon: String,
    #[serde(flatten)]
    #[allow(dead_code)]
    extra: HashMap<String, serde_json::Value>,
}

pub struct WeatherApi {
    api_key: String,
    base_url: String,
}

impl WeatherApi {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            base_url: "https://api.weatherapi.com".to_string(),
        }
    }

    #[cfg(test)]
    pub fn with_base_url(mut self, base_url: String) -> Self {
        self.base_url = base_url;
        self
    }

    async fn get_weather(
        &self,
        location: &str,
        date: Option<NaiveDate>,
    ) -> Result<WeatherResponse, ProviderError> {
        if location.is_empty() {
            return Err(ProviderError::InvalidLocation(location.to_string()));
        }

        if self.api_key.is_empty() {
            return Err(ProviderError::InvalidApiKey("WeatherApi".to_string()));
        }

        let url = if let Some(date) = date {
            format!(
                "{}/v1/current.json?key={}&q={}&aqi=no&unixend_dt={}",
                self.base_url, self.api_key, location, date
            )
        } else {
            format!(
                "{}/v1/current.json?key={}&q={}&aqi=no",
                self.base_url, self.api_key, location
            )
        };

        let res = reqwest::get(&url).await?;
        let weather_response: WeatherResponse = res.json().await?;

        Ok(weather_response)
    }
}

#[async_trait::async_trait]
impl WeatherProvider for WeatherApi {
    async fn fetch(
        &self,
        location: &str,
        date: Option<NaiveDate>,
    ) -> Result<WeatherData, ProviderError> {
        let weather_response = self.get_weather(location, date).await?;

        let wind_kph = weather_response
            .current
            .extra
            .get("wind_kph")
            .and_then(|v| v.as_f64());

        Ok(WeatherData {
            location: location.to_string(),
            timestamp: weather_response.current.last_updated.clone(),
            temp_c: weather_response.current.temp_c,
            condition: weather_response.current.condition.text,
            wind_kph,
        })
    }
}
