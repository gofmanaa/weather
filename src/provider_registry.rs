use crate::errors::AppError;
use crate::weather_providers::openweather::OpenWeather;
use crate::weather_providers::weatherapi::WeatherApi;
use crate::weather_providers::WeatherProvider;
use dotenvy::var;
use std::{collections::HashMap, sync::Arc};
use tracing::{info, warn};

pub trait ProviderLookup {
    fn lookup(&self, name: &str) -> Option<Arc<dyn WeatherProvider>>;
    fn list(&self) -> Vec<String>;
}

pub struct ProviderRegistry {
    providers: HashMap<String, Arc<dyn WeatherProvider>>,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }

    pub fn register<T>(&mut self, name: &str, provider: T)
    where
        T: WeatherProvider + 'static,
    {
        self.providers.insert(name.to_string(), Arc::new(provider));
    }

    pub fn get(&self, name: &str) -> Option<Arc<dyn WeatherProvider>> {
        self.providers.get(name).cloned()
    }

    pub fn list_providers(&self) -> Vec<String> {
        self.providers.keys().cloned().collect()
    }
}

impl ProviderLookup for ProviderRegistry {
    fn lookup(&self, name: &str) -> Option<Arc<dyn WeatherProvider>> {
        self.get(name)
    }

    fn list(&self) -> Vec<String> {
        self.list_providers()
    }
}

pub fn build_registry() -> Result<ProviderRegistry, AppError> {
    let mut reg = ProviderRegistry::new();

    if let Ok(api_key) = var("OPENWEATHER_API_KEY") {
        reg.register("openweather", OpenWeather::new(api_key));
        info!("OpenWeather registered");
    } else {
        warn!("Skipping OpenWeather provider: OPENWEATHER_API_KEY not set");
    }

    if let Ok(api_key) = var("WEATHERAPI_API_KEY") {
        reg.register("weatherapi", WeatherApi::new(api_key));
        info!("WeatherApi registered");
    } else {
        warn!("Skipping WeatherApi provider: WEATHERAPI_API_KEY not set");
    }

    if reg.list().is_empty() {
        return Err(AppError::MissingApiKey("Provide Api Key ".to_string()));
    }

    Ok(reg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::weather_providers::error::ProviderError;
    use crate::weather_providers::WeatherData;
    use async_trait::async_trait;
    use chrono::NaiveDate;

    #[derive(Debug, PartialEq)]
    struct MockProvider {}

    #[async_trait]
    impl WeatherProvider for MockProvider {
        async fn fetch(
            &self,
            location: &str,
            date: Option<NaiveDate>,
        ) -> Result<WeatherData, ProviderError> {
            Ok(WeatherData {
                location: location.to_string(),
                datetime: date.unwrap().to_string(),
                temp_c: 0.0,
                humidity: 0.0,
                pressure: 0.0,
                condition: "".to_string(),
                wind_kph: 0.0,
                wind_deg: 0.0,
            })
        }
    }
    #[test]
    fn register_providers() {
        let mut reg = ProviderRegistry::new();
        reg.register("provider1", MockProvider {});
        reg.register("provider2", MockProvider {});
        assert_eq!(reg.providers.len(), 2);
    }
    #[test]
    fn lookup_missing_provider() {
        let reg = ProviderRegistry::new();
        assert!(reg.lookup("unknown").is_none());
    }

    #[tokio::test]
    async fn lookup_returns_correct_provider() {
        let mut reg = ProviderRegistry::new();
        reg.register("mock", MockProvider {});

        let provider = reg.lookup("mock").unwrap();

        let date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        let result = provider.fetch("London", Some(date)).await.unwrap();

        assert_eq!(result.location, "London");
        assert_eq!(result.datetime, "2026-01-01");
    }

    #[test]
    fn registry_handles_multiple_providers() {
        let mut reg = ProviderRegistry::new();
        reg.register("p1", MockProvider {});
        reg.register("p2", MockProvider {});

        assert!(reg.lookup("p1").is_some());
        assert!(reg.lookup("p2").is_some());
    }
}
