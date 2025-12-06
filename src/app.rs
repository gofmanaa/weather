use crate::errors::AppError;
use crate::provider_registry::ProviderRegistry;
use crate::weather_providers::WeatherData;
use chrono::NaiveDateTime;

/// App for querying weather providers.
pub struct WeatherApp {
    registry: ProviderRegistry,
}

impl WeatherApp {
    pub fn new(manager: ProviderRegistry) -> Self {
        Self { registry: manager }
    }

    /// Fetch weather for a provider, location, and optional date.
    pub async fn run(
        &self,
        provider_name: &str,
        location: &str,
        date: Option<NaiveDateTime>,
    ) -> Result<WeatherData, AppError> {
        let Some(provider) = self.registry.get(provider_name) else {
            return Err(AppError::InvalidProvider(format!(
                "Provider '{provider_name}' not found"
            )));
        };

        provider
            .fetch(location, date)
            .await
            .map_err(|e| AppError::InvalidDate(format!("Failed to fetch weather: {e}")))
    }

    /// Check if a provider exists.
    pub fn provider_exist(&self, name: &str) -> bool {
        self.registry.get(name).is_some()
    }

    /// List all provider names.
    pub fn list(&self) -> Vec<String> {
        self.registry.list_providers()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::weather_providers::WeatherProvider;
    use crate::weather_providers::error::ProviderError;
    use async_trait::async_trait;

    #[tokio::test]
    async fn weather_app_empty_registry() {
        let wapp = WeatherApp::new(ProviderRegistry::new());
        assert!(wapp.list().is_empty());

        let res = wapp.run("", "location", None).await;
        assert!(res.is_err());
    }

    struct MockProvider;

    #[async_trait]
    impl WeatherProvider for MockProvider {
        async fn fetch(
            &self,
            _location: &str,
            _date: Option<NaiveDateTime>,
        ) -> Result<WeatherData, ProviderError> {
            Ok(WeatherData::default())
        }
    }
    #[tokio::test]
    async fn weather_app() {
        let mut register = ProviderRegistry::new();
        register.register("something", MockProvider);
        let wapp = WeatherApp::new(register);
        assert!(!wapp.list().is_empty());

        let res = wapp.run("something", "location", None).await;
        assert!(res.is_ok());
    }
}
