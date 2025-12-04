use crate::config::SettingsError;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Configuration error: {0}")]
    Config(SettingsError),

    #[error("Provider error: {0}")]
    Provider(String),

    #[error("No provider configured")]
    NoProvider,

    #[error("Invalid provider name: {0}")]
    InvalidProvider(String),

    #[error("Invalid date: {0}")]
    InvalidDate(String),
}
