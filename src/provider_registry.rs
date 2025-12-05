use crate::config::Settings;
use crate::errors::AppError;
use crate::weather_providers::WeatherProvider;
use crate::weather_providers::openweather::OpenWeather;
use crate::weather_providers::weatherapi::WeatherApi;
use std::{collections::HashMap, sync::Arc};
use tracing::{error, info, warn};

/// Holds registered weather providers.
pub struct ProviderRegistry {
    providers: HashMap<String, Arc<dyn WeatherProvider>>,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }

    /// Register a provider by name.
    pub fn register<P>(&mut self, name: &str, provider: P)
    where
        P: WeatherProvider + 'static,
    {
        if self.providers.contains_key(name) {
            error!(
                "Warning: Provider '{}' is already registered and will be overwritten",
                name
            );
        }
        self.providers.insert(name.to_string(), Arc::new(provider));
    }

    /// Get a provider by name.
    pub fn get(&self, name: &str) -> Option<Arc<dyn WeatherProvider>> {
        self.providers.get(name).cloned()
    }

    /// List all registered provider names.
    pub fn list_providers(&self) -> Vec<String> {
        let mut keys: Vec<_> = self.providers.keys().cloned().collect();
        keys.sort();
        keys
    }
}

/// Build a registry from settings.
pub fn build_registry(settings: &Settings) -> Result<ProviderRegistry, AppError> {
    let mut registry = ProviderRegistry::new();

    for name in settings.providers.keys() {
        match name.as_str() {
            "openweather" => {
                registry.register(
                    name,
                    OpenWeather::new(settings.get_api_key(name))
                        .map_err(|e| AppError::MissingApiKey(e.to_string()))?,
                );
                info!("OpenWeather registered");
            },
            "weatherapi" => {
                registry.register(
                    name,
                    WeatherApi::new(settings.get_api_key(name))
                        .map_err(|e| AppError::MissingApiKey(e.to_string()))?,
                );
                info!("WeatherApi registered");
            },
            _ => warn!("Provider `{}` in config is not implemented", name),
        }
    }

    if registry.list_providers().is_empty() {
        return Err(AppError::MissingApiKey(
            "No valid providers configured".to_string(),
        ));
    }

    Ok(registry)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::weather_providers::WeatherData;
    use crate::weather_providers::error::ProviderError;
    use async_trait::async_trait;
    use chrono::{DateTime, Local, NaiveDate, TimeZone, Utc};

    #[derive(Debug, PartialEq)]
    struct MockProvider {}

    #[async_trait]
    impl WeatherProvider for MockProvider {
        async fn fetch(
            &self,
            location: &str,
            date: Option<NaiveDate>,
        ) -> Result<WeatherData, ProviderError> {
            let datetime = date
                .map(|d| {
                    let ndt = d.and_hms_opt(12, 13, 0).unwrap();
                    DateTime::<Utc>::from_naive_utc_and_offset(ndt, Utc)
                })
                .unwrap();

            Ok(WeatherData {
                location: location.to_string(),
                datetime,
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
        assert!(reg.get("unknown").is_none());
    }

    #[tokio::test]
    async fn get_returns_correct_provider() {
        let mut reg = ProviderRegistry::new();
        reg.register("mock", MockProvider {});

        let provider = reg.get("mock").unwrap();

        let date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        let result = provider.fetch("London", Some(date)).await.unwrap();

        assert_eq!(result.location, "London");

        let expected_datetime = Local.with_ymd_and_hms(2026, 1, 1, 12, 13, 0).unwrap();

        assert_eq!(
            result.datetime.with_timezone(&Utc),
            expected_datetime.with_timezone(&Utc)
        );
    }

    #[test]
    fn registry_handles_multiple_providers() {
        let mut reg = ProviderRegistry::new();
        reg.register("p1", MockProvider {});
        reg.register("p2", MockProvider {});

        assert!(reg.get("p1").is_some());
        assert!(reg.get("p2").is_some());
    }
}
