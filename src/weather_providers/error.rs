use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProviderError {
    #[error("API request failed: {0}")]
    Request(#[from] reqwest::Error),

    #[error("Failed to parse API response: {0}")]
    Parse(#[from] serde_json::Error),

    #[error("API key is missing or invalid for {0}")]
    InvalidApiKey(String),

    #[error("Location '{0}' is invalid or not found")]
    InvalidLocation(String),

    #[error("An unexpected error occurred: {0}")]
    Unexpected(String),
}