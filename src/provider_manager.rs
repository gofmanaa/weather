use crate::provider_registry::ProviderLookup;
use crate::weather_providers::WeatherProvider;
use std::sync::Arc;

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

    pub fn list(&self) -> Vec<String> {
        self.lookup.list()
    }
}

#[cfg(test)]
mod tests {
    use crate::app::WeatherApp;
    use crate::provider_manager::ProviderManager;
    use crate::provider_registry::ProviderRegistry;
    use crate::weather_providers::error::ProviderError;
    use crate::weather_providers::{WeatherData, WeatherProvider};
    use async_trait::async_trait;
    use chrono::{Local, NaiveDate};

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
                datetime: "".to_string(),
                temp_c: 25.0,
                humidity: 0.0,
                pressure: 0.0,
                condition: "Sunny".to_string(),
                wind_kph: 0.0,
                wind_deg: 0.0,
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
        assert_eq!(weather.unwrap().temp_c, 25.0);
    }
}
