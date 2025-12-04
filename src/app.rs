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
        provider: &str,
        city: &str,
        date: Option<NaiveDate>,
    ) -> Result<WeatherData, AppError> {
        let provider = match self.manager.get(provider) {
            Some(provider) => provider,
            None => Err(AppError::InvalidProvider(format!(
                "Provider '{}' not found",
                provider
            )))?,
        };

        provider
            .fetch(city, date)
            .await
            .map_err(|e| AppError::InvalidDate(e.to_string()))
    }
}
