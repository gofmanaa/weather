use crate::errors::AppError;
use crate::provider_registry::ProviderLookup;
use crate::weather_providers::WeatherProvider;
use std::sync::Arc;

#[derive(Debug, Clone, Default, PartialEq)]
#[non_exhaustive]
pub enum ProviderKind {
    OpenWeather,
    #[default]
    WeatherApi,
}

impl ProviderKind {
    pub fn name(&self) -> String {
        match self {
            ProviderKind::OpenWeather => "openweather".to_string(),
            ProviderKind::WeatherApi => "weatherapi".to_string(),
        }
    }

    pub fn from_str(name: &str) -> Result<Self, AppError> {
        match name.to_lowercase().as_str() {
            "openweather" => Ok(ProviderKind::OpenWeather),
            "weatherapi" => Ok(ProviderKind::WeatherApi),
            _ => Err(AppError::InvalidProvider(name.to_string())),
        }
    }
}

pub struct ProviderManager<L> {
    lookup: L,
}

impl<L> ProviderManager<L>
where
    L: ProviderLookup,
{
    pub fn new(lookup: L) -> Self {
        Self { lookup }
    }

    pub fn get(&self, name: &str) -> Option<Arc<dyn WeatherProvider>> {
        self.lookup.lookup(name)
    }

    // pub fn from_str(provider_name: &str) -> Result<Self, AppError> {
    //     let provider_kind = ProviderKind::from_str(provider_name)?;
    //     let provider: Arc<dyn WeatherProvider> = match provider_kind {
    //         ProviderKind::OpenWeather => {
    //             let api_key = try_apikey_from_env(provider_name)
    //                 .map_err(|e| AppError::InvalidProvider(e.to_string()))?;
    //             let provider = OpenWeather::new(api_key);
    //             Arc::new(provider)
    //         }
    //         ProviderKind::WeatherApi => {
    //             let api_key = try_apikey_from_env(provider_name)
    //                 .map_err(|e| AppError::InvalidProvider(e.to_string()))?;
    //             let provider = WeatherApi::new(api_key);
    //             Arc::new(provider)
    //         }
    //     };
    //
    //     Ok(Self { provider })
    // }

    // pub async fn get_weather(
    //     &self,
    //     location: String,
    //     date: Option<DateTime<Local>>,
    // ) -> Result<Weather, ProviderError> {
    //     self.provider.fetch(&location, date).await
    // }
}

#[cfg(test)]
mod tests {
    use crate::app::WeatherApp;
    use crate::provider_manager::{ProviderKind, ProviderManager};
    use crate::provider_registry::ProviderRegistry;
    use crate::weather_providers::error::ProviderError;
    use crate::weather_providers::{WeatherData, WeatherProvider};
    use async_trait::async_trait;
    use chrono::{Local, NaiveDate};

    #[test]
    fn provider_from() {
        let p = ProviderKind::from_str("OpenWeather");
        assert_eq!(p.unwrap(), ProviderKind::OpenWeather);
        let p = ProviderKind::from_str("weatherapi");
        assert_eq!(p.unwrap(), ProviderKind::WeatherApi);
    }

    #[test]
    fn provider_name() {
        assert_eq!(ProviderKind::OpenWeather.name(), "openweather");
        assert_eq!(ProviderKind::WeatherApi.name(), "weatherapi");
    }

    #[derive(Debug, PartialEq)]
    pub struct MockWeatherApi;

    #[async_trait]
    impl WeatherProvider for MockWeatherApi {
        async fn fetch(
            &self,
            location: &str,
            date: Option<NaiveDate>,
        ) -> Result<WeatherData, ProviderError> {
            let dt_str = date.map_or("now".to_string(), |d| d.to_string());
            Ok(WeatherData {
                location: format!("{} (MockProvider at {})", location, dt_str),
                timestamp: "".to_string(),
                temp_c: 25.0,
                condition: "Sunny".to_string(),
                wind_kph: None,
            })
        }
    }

    #[tokio::test]
    async fn provider_manager() {
        let provider_name = "mock_provider";
        let mut registry = ProviderRegistry::new();
        registry.register(provider_name, MockWeatherApi {});

        let manager = ProviderManager::new(registry);
        let app = WeatherApp::new(manager);

        let location = String::from("address");
        let date = Local::now().date_naive();

        let weather = app.run(provider_name, &location, Some(date)).await;

        assert!(weather.is_ok());
    }
}
