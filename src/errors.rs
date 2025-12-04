use crate::config::SettingsError;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Configuration error: {0}")]
    Config(SettingsError),

    #[error("Invalid provider name: {0}")]
    InvalidProvider(String),

    #[error("Invalid date: {0}")]
    InvalidDate(String),

    #[error("Missing API key: {0}")]
    MissingApiKey(String),
}
