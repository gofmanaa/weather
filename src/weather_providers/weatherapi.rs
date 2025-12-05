use crate::weather_providers::error::ProviderError;
use crate::weather_providers::{WeatherData, WeatherProvider};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::debug;

// History weather API method returns historical weather for a date on or after 1st Jan, 2010 as json. The data is returned as a Forecast Object.
// https://api.weatherapi.com/v1/history.json?q=Porto%2CPT&dt=2025-12-1&key=******

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherResponse {
    pub current: Option<WeatherCondition>,
    pub forecast: Option<Forecast>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Forecast {
    pub forecastday: Vec<ForecastDay>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecastDay {
    pub date: String,
    pub day: DayCondition,
    pub hour: Vec<HourCondition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DayCondition {
    pub avgtemp_c: f64,
    pub avghumidity: f64,
    pub maxwind_kph: f64,
    pub condition: ConditionFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HourCondition {
    pub time: String,
    pub temp_c: f64,
    pub wind_kph: f64,
    pub wind_degree: f64,
    pub humidity: f64,
    pub pressure_mb: f64,
    pub condition: ConditionFields,
    #[serde(flatten)]
    #[allow(dead_code)]
    extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherCondition {
    pub last_updated: String,
    pub temp_c: f64,
    pub condition: ConditionFields,
    pub wind_kph: f64,
    pub wind_degree: f64,
    pub humidity: f64,
    pub pressure_mb: f64,
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

impl From<WeatherResponse> for WeatherData {
    fn from(resp: WeatherResponse) -> Self {
        if let Some(current) = resp.current {
            WeatherData {
                location: String::new(),
                datetime: current.last_updated,
                temp_c: current.temp_c,
                humidity: current.humidity,
                pressure: current.pressure_mb,
                condition: current.condition.text,
                wind_kph: current.wind_kph,
                wind_deg: current.wind_degree,
            }
        } else if let Some(forecast) = resp.forecast {
            let forecast_day = &forecast.forecastday[0];
            let day = &forecast_day.day;
            let first_hour = &forecast_day.hour[0];

            WeatherData {
                location: String::new(),
                datetime: forecast_day.date.clone(),
                temp_c: day.avgtemp_c,
                humidity: day.avghumidity,
                pressure: first_hour.pressure_mb,
                condition: day.condition.text.clone(),
                wind_kph: day.maxwind_kph,
                wind_deg: first_hour.wind_degree,
            }
        } else {
            panic!("WeatherResponse has neither current nor forecast");
        }
    }
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
        debug!("weatherapi location: {}, date: {:?}", location, date);

        if location.is_empty() {
            return Err(ProviderError::InvalidLocation(location.to_string()));
        }

        if self.api_key.is_empty() {
            return Err(ProviderError::InvalidApiKey("WeatherApi".to_string()));
        }

        let url = date.map_or_else(
            || {
                format!(
                    "{}/v1/current.json?key={}&q={}&aqi=no",
                    self.base_url, self.api_key, location
                )
            },
            |date| {
                // Date on or after 1st Jan, 2015 in yyyy-MM-dd format
                format!(
                    "{}/v1/history.json?key={}&q={}&aqi=no&dt={}",
                    self.base_url, self.api_key, location, date
                )
            },
        );

        let res = reqwest::get(&url).await.map_err(ProviderError::Request)?;
        debug!("Status :{:#?}", res.status());

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
        let weather = self.get_weather(location, date).await?;
        let mut res = WeatherData::from(weather);
        res.location = location.to_string();

        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    const MOCK_RESPONSE: &str = r#"{
      "current": {
        "last_updated": "2025-12-03 14:15",
        "temp_c": 28.2,
        "wind_kph": 18.0,
        "wind_degree": 286.0,
        "humidity": 67.0,
        "pressure_mb": 1019.0,
        "condition": {
          "text": "Sunny",
          "icon": "//cdn.weatherapi.com/weather/64x64/day/113.png"
        }
      }
    }"#;

    #[tokio::test]
    async fn success_parsing() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/v1/current.json"))
            .respond_with(
                ResponseTemplate::new(200).set_body_raw(MOCK_RESPONSE, "application/json"),
            )
            .mount(&server)
            .await;

        let api = WeatherApi::new("key123".into()).with_base_url(server.uri());
        let response = api.get_weather("Porto Alegre", None).await.unwrap();
        let result = WeatherData::from(response);

        assert_eq!(result.temp_c, 28.2);
        assert_eq!(result.wind_kph, 18.0);
        assert_eq!(result.condition, "Sunny");
        assert_eq!(result.datetime, "2025-12-03 14:15");
    }

    #[tokio::test]
    async fn invalid_location() {
        let api = WeatherApi::new("key123".into());
        let result = api.get_weather("", None).await;

        match result {
            Err(ProviderError::InvalidLocation(_)) => {},
            _ => panic!("expected InvalidLocation error"),
        }
    }

    #[tokio::test]
    async fn missing_api_key() {
        let api = WeatherApi::new("".into());
        let result = api.get_weather("Porto Alegre", None).await;

        match result {
            Err(ProviderError::InvalidApiKey(_)) => {},
            _ => panic!("expected InvalidApiKey error"),
        }
    }

    #[tokio::test]
    async fn fetch_provider() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/v1/current.json"))
            .respond_with(
                ResponseTemplate::new(200).set_body_raw(MOCK_RESPONSE, "application/json"),
            )
            .mount(&server)
            .await;

        let api = WeatherApi::new("abc".into()).with_base_url(server.uri());
        let result = api.fetch("Porto Alegre", None).await.unwrap();

        assert_eq!(result.location, "Porto Alegre");
        assert_eq!(result.temp_c, 28.2);
        assert_eq!(result.wind_kph, 18.0);
        assert_eq!(result.condition, "Sunny");
        assert_eq!(result.datetime, "2025-12-03 14:15");
    }
}
