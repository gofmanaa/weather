use crate::errors::AppError;
use crate::provider_registry::ProviderRegistry;
use crate::weather_providers::WeatherData;
use chrono::NaiveDate;

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
        date: Option<NaiveDate>,
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
