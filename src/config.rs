use config::{Config, Environment, File};
use dotenvy::var;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::{fs, path::PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum SettingsError {
    #[error("Failed to load settings: {0}")]
    Load(#[from] config::ConfigError),
    #[error("Failed to load .env file: {0}")]
    Env(#[from] dotenvy::Error),
    #[error("Failed to save settings: {0}")]
    Save(String),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Settings {
    pub default_provider: String,
}

pub fn load_settings(config_path: &Path) -> Result<Settings, SettingsError> {
    dotenvy::dotenv().ok();

    let mut builder =
        Config::builder().add_source(File::from(PathBuf::from(config_path)).required(false));

    if let Ok(provider) = dotenvy::var("DEFAULT_PROVIDER") {
        builder = builder.set_override("default_provider", provider)?;
    }

    let config = builder
        .add_source(
            Environment::with_prefix("CONF")
                .separator("_")
                .try_parsing(true),
        )
        .build()?;

    let settings = config
        .try_deserialize::<Settings>()
        .map_err(SettingsError::Load)?;

    Ok(settings)
}

pub fn save_settings(settings: &Settings, path: PathBuf) -> Result<(), SettingsError> {
    let toml_settings =
        toml::to_string_pretty(settings).map_err(|e| SettingsError::Save(e.to_string()))?;
    fs::write(path, toml_settings).map_err(|e| SettingsError::Save(e.to_string()))?;

    Ok(())
}

pub fn try_apikey_from_env(provider_name: &str) -> Result<String, SettingsError> {
    var(format!("{}_API_KEY", provider_name)).map_err(SettingsError::Env)
}
