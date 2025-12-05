use config::{Config, File};
use dotenvy::var;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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
pub struct ProviderSettings {
    pub api_key: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Settings {
    pub default_provider: String,
    pub providers: HashMap<String, ProviderSettings>,
}

impl Settings {
    pub fn get_api_key(&self, provider_name: &str) -> Option<String> {
        let env_var = format!("{}_API_KEY", provider_name.to_uppercase());
        if let Ok(key) = var(&env_var) {
            return Some(key);
        }
        self.providers.get(provider_name).map(|p| p.api_key.clone())
    }
}

pub fn load_settings(config_path: &Path) -> Result<Settings, SettingsError> {
    let mut builder = Config::builder();

    builder = builder.set_default("default_provider", "weatherapi")?;

    if config_path.exists() {
        builder = builder.add_source(File::from(PathBuf::from(config_path)).required(false));
    }

    if let Ok(provider) = var("DEFAULT_PROVIDER") {
        builder = builder.set_override("default_provider", provider)?;
    }

    let config = builder.build()?;
    let settings = config
        .try_deserialize::<Settings>()
        .map_err(SettingsError::Load)?;

    Ok(settings)
}

pub fn save_settings(settings: &Settings, path: &PathBuf) -> Result<(), SettingsError> {
    let toml_settings =
        toml::to_string_pretty(settings).map_err(|e| SettingsError::Save(e.to_string()))?;
    fs::write(path, toml_settings).map_err(|e| SettingsError::Save(e.to_string()))?;

    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::fs;
    use std::io::Write;
    use std::path::Path;

    #[test]
    #[serial]
    fn test_load_settings_no_file() {
        let test_provider_name = "test_provider";

        temp_env::with_var("DEFAULT_PROVIDER", Some(test_provider_name), || {
            let settings = Settings {
                default_provider: test_provider_name.to_string(),
                providers: {
                    let mut m = HashMap::new();
                    m.insert(
                        test_provider_name.to_string(),
                        ProviderSettings {
                            api_key: "dummy".to_string(),
                        },
                    );
                    m
                },
            };

            let tmp_path = Path::new("tests/tmp_no_file.toml");
            let toml_data = toml::to_string(&settings).unwrap();
            fs::write(tmp_path, toml_data).unwrap();

            let s = load_settings(tmp_path).unwrap();
            assert_eq!(s.default_provider, test_provider_name);

            fs::remove_file(tmp_path).unwrap();
        });
    }

    #[test]
    #[serial]
    fn test_load_settings_with_dotenv() {
        let env_path = Path::new("tests/.env.test");
        fs::create_dir_all("tests").unwrap();
        let test_provider_name = "test_openweather";
        {
            let mut file = fs::File::create(env_path).unwrap();
            writeln!(file, "DEFAULT_PROVIDER={}", test_provider_name).unwrap();
        }

        temp_env::with_var("DEFAULT_PROVIDER", None::<String>, || {
            dotenvy::from_filename(env_path).unwrap();

            let tmp_path = Path::new("tests/tmp_dotenv.toml");
            let mut providers = HashMap::new();
            providers.insert(
                test_provider_name.to_string(),
                ProviderSettings {
                    api_key: "dummy".to_string(),
                },
            );
            let settings = Settings {
                default_provider: test_provider_name.to_string(),
                providers,
            };
            let toml_data = toml::to_string(&settings).unwrap();
            fs::write(tmp_path, toml_data).unwrap();

            let s = load_settings(tmp_path).unwrap();
            assert_eq!(s.default_provider, test_provider_name);

            fs::remove_file(tmp_path).unwrap();
            fs::remove_file(env_path).unwrap();
        });
    }

    #[test]
    #[serial]
    fn test_load_settings_from_file() {
        let settings_path = Path::new("tests/settings_test.toml");
        fs::create_dir_all("tests").unwrap();
        let test_provider_name = "test_from_storage_provider";

        {
            let mut file = fs::File::create(settings_path).unwrap();
            writeln!(file, r#"default_provider = "{}""#, test_provider_name).unwrap();
            writeln!(file, r#"[providers.{}]"#, test_provider_name).unwrap();
            writeln!(file, r#"api_key = "dummy_api_key""#).unwrap();
        }

        let s = load_settings(settings_path).unwrap();
        assert_eq!(s.default_provider, test_provider_name);

        fs::remove_file(settings_path).unwrap();
    }
}
