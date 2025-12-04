use crate::errors::AppError;
use crate::provider_manager::ProviderManager;
use crate::provider_registry::ProviderLookup;
use crate::weather_providers::WeatherData;
use chrono::NaiveDate;

pub struct WeatherApp<L> {
    manager: ProviderManager<L>,
}

impl<L> WeatherApp<L>
where
    L: ProviderLookup,
{
    pub fn new(manager: ProviderManager<L>) -> Self {
        Self { manager }
    }

    pub async fn run(
        &self,
        provider_name: &str,
        city: &str,
        date: Option<NaiveDate>,
    ) -> Result<WeatherData, AppError> {
        let Some(provider) = self.manager.get(provider_name) else {
            return Err(AppError::InvalidProvider(format!(
                "Provider '{provider_name}' not found"
            )));
        };

        provider
            .fetch(city, date)
            .await
            .map_err(|e| AppError::InvalidDate(e.to_string()))
    }

    pub fn provider_exist(&self, name: &str) -> bool {
        self.manager.get(name).is_some()
    }

    pub fn list(&self) -> Vec<String> {
        self.manager.list()
    }
}
