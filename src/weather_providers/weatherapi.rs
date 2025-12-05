use crate::weather_providers::error::ProviderError;
use crate::weather_providers::{WeatherData, WeatherProvider};
use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::TryFrom;
use tracing::debug;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum WeatherResponse {
    Current {
        location: Location,
        current: WeatherCondition,
    },
    History {
        location: Location,
        forecast: Forecast,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Forecast {
    pub forecastday: Vec<ForecastDay>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub name: String,
    pub region: String,
    pub country: String,
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionFields {
    pub text: String,
    pub icon: String,
    #[serde(flatten)]
    #[allow(dead_code)]
    extra: HashMap<String, serde_json::Value>,
}

impl TryFrom<WeatherResponse> for WeatherData {
    type Error = ProviderError;

    fn try_from(resp: WeatherResponse) -> Result<Self, Self::Error> {
        match resp {
            WeatherResponse::Current { current, location } => {
                let datetime = parse_local_datetime(&current.last_updated)?;
                let location = format!("{}, {}", location.name, location.country);

                Ok(WeatherData {
                    location,
                    datetime,
                    temp_c: current.temp_c,
                    humidity: current.humidity,
                    pressure: current.pressure_mb,
                    condition: current.condition.text,
                    wind_kph: current.wind_kph,
                    wind_deg: current.wind_degree,
                })
            },

            WeatherResponse::History { forecast, location } => {
                let forecast_day = &forecast.forecastday[0];
                let day = &forecast_day.day;

                let first_hour = &forecast_day.hour[0];
                let datetime = parse_local_datetime(&first_hour.time)?;
                let location = format!("{}, {}", location.name, location.country);

                Ok(WeatherData {
                    location,
                    datetime,
                    temp_c: day.avgtemp_c,
                    humidity: day.avghumidity,
                    pressure: first_hour.pressure_mb,
                    condition: day.condition.text.clone(),
                    wind_kph: first_hour.wind_kph,
                    wind_deg: first_hour.wind_degree,
                })
            },
        }
    }
}

/// Parse "YYYY-MM-DD HH:MM" string into `DateTime<Utc>`
fn parse_local_datetime(date_str: &str) -> Result<DateTime<Utc>, ProviderError> {
    let naive = NaiveDateTime::parse_from_str(date_str, "%Y-%m-%d %H:%M")
        .map_err(|e| ProviderError::ParseDateTime(format!("Failed to parse datetime: {e}")))?;
    Ok(DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc))
}

pub struct WeatherApi {
    api_key: String,
    base_url: Url,
}

impl WeatherApi {
    pub fn new(api_key: Option<String>) -> Result<Self, ProviderError> {
        let base_url = Url::parse("https://api.weatherapi.com")
            .map_err(|e| ProviderError::Error(format!("Invalid API URL: {e}")))?;

        let api_key = api_key.ok_or_else(|| {
            ProviderError::InvalidApiKey("WeatherApi requires API_KEY".to_string())
        })?;

        Ok(Self { api_key, base_url })
    }

    #[cfg(test)]
    pub fn with_base_url(mut self, base_url: impl Into<Url>) -> Self {
        self.base_url = base_url.into();
        self
    }

    async fn get_weather(
        &self,
        location: impl AsRef<str>,
        date: Option<NaiveDate>,
    ) -> Result<WeatherResponse, ProviderError> {
        debug!(
            "weatherapi location: {}, date: {:?}",
            location.as_ref(),
            date
        );

        if location.as_ref().is_empty() {
            return Err(ProviderError::InvalidLocation(
                location.as_ref().to_string(),
            ));
        }

        let url = date.map_or_else(
            || {
                format!(
                    "{}v1/current.json?key={}&q={}&aqi=no",
                    self.base_url,
                    self.api_key,
                    location.as_ref()
                )
            },
            |date| {
                format!(
                    "{}v1/history.json?key={}&q={}&aqi=no&dt={}",
                    self.base_url,
                    self.api_key,
                    location.as_ref(),
                    date
                )
            },
        );

        let res = reqwest::get(&url)
            .await
            .map_err(ProviderError::Request)?
            .error_for_status()
            .map_err(ProviderError::Request)?;

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
        let res = WeatherData::try_from(weather)
            .map_err(|e| ProviderError::ParseDateTime(e.to_string()))?;

        Ok(WeatherData { ..res })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    const MOCK_CURRENT_RESPONSE: &str = r#"{
        "location": {
            "name": "Porto",
            "region": "Porto",
            "country": "Portugal",
            "lat": 41.15,
            "lon": -8.6167,
            "tz_id": "Europe/Lisbon",
            "localtime_epoch": 1764955303,
            "localtime": "2025-12-05 17:21"
        },
        "current": {
            "last_updated_epoch": 1764954900,
            "last_updated": "2025-12-05 17:15",
            "temp_c": 16.1,
            "temp_f": 61.0,
            "is_day": 0,
            "condition": {
                "text": "Partly cloudy",
                "icon": "//cdn.weatherapi.com/weather/64x64/night/116.png",
                "code": 1003
            },
            "wind_mph": 13.6,
            "wind_kph": 22.0,
            "wind_degree": 245,
            "wind_dir": "WSW",
            "pressure_mb": 1018.0,
            "pressure_in": 30.06,
            "precip_mm": 0.81,
            "precip_in": 0.03,
            "humidity": 94,
            "cloud": 75,
            "feelslike_c": 16.1,
            "feelslike_f": 61.0,
            "windchill_c": 15.6,
            "windchill_f": 60.1,
            "heatindex_c": 15.6,
            "heatindex_f": 60.1,
            "dewpoint_c": 14.8,
            "dewpoint_f": 58.6,
            "vis_km": 9.0,
            "vis_miles": 5.0,
            "uv": 0.0,
            "gust_mph": 24.9,
            "gust_kph": 40.1,
            "short_rad": 0,
            "diff_rad": 0,
            "dni": 0,
            "gti": 0
        }
    }"#;

    #[test]
    fn test_mock_json() {
        let resp = serde_json::from_str::<WeatherResponse>(MOCK_CURRENT_RESPONSE);
        assert!(resp.is_ok(), "Failed to parse JSON: {:?}", resp.err());

        if let WeatherResponse::Current { location, current } = resp.unwrap() {
            assert_eq!(location.name, "Porto");
            assert_eq!(current.temp_c, 16.1);
            assert_eq!(current.condition.text, "Partly cloudy");
        } else {
            panic!("Expected WeatherResponse::Current variant");
        }
    }

    #[tokio::test]
    async fn success_parsing() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/v1/current.json"))
            .respond_with(
                ResponseTemplate::new(200).set_body_raw(MOCK_CURRENT_RESPONSE, "application/json"),
            )
            .mount(&server)
            .await;

        let api = WeatherApi::new(Some("test_api_key".to_string()))
            .unwrap()
            .with_base_url(server.uri().parse::<Url>().unwrap());
        let response = api.get_weather("Porto Alegre", None).await.unwrap();
        let result = WeatherData::try_from(response).unwrap();

        let expected_datetime = DateTime::parse_from_rfc3339("2025-12-05T17:15:00+00:00")
            .unwrap()
            .with_timezone(&Utc);

        assert_eq!(result.temp_c, 16.1);
        assert_eq!(result.wind_kph, 22.0);
        assert_eq!(result.condition, "Partly cloudy");
        assert_eq!(result.datetime, expected_datetime);
    }

    #[tokio::test]
    async fn invalid_location() {
        let api = WeatherApi::new(Some("test_api_key".to_string())).unwrap();
        let result = api.get_weather("", None).await;

        match result {
            Err(ProviderError::InvalidLocation(_)) => {},
            _ => panic!("expected InvalidLocation error"),
        }
    }

    #[tokio::test]
    async fn missing_api_key() {
        let api = WeatherApi::new(None);

        match api {
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
                ResponseTemplate::new(200).set_body_raw(MOCK_CURRENT_RESPONSE, "application/json"),
            )
            .mount(&server)
            .await;

        let api = WeatherApi::new(Some("test_api_key".to_string()))
            .unwrap()
            .with_base_url(server.uri().parse::<Url>().unwrap());
        let result = api.fetch("Porto,PT", None).await.unwrap();

        let expected_datetime = DateTime::parse_from_rfc3339("2025-12-05T17:15:00+00:00")
            .unwrap()
            .with_timezone(&Utc);

        assert_eq!(result.location, "Porto, Portugal");
        assert_eq!(result.temp_c, 16.1);
        assert_eq!(result.wind_kph, 22.0);
        assert_eq!(result.condition, "Partly cloudy");
        assert_eq!(result.datetime, expected_datetime);
    }
}
