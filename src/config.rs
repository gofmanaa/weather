use config::{Config, File};
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
    use std::io::Write;
    #[test]
    #[serial_test::serial]
    fn test_load_settings_no_file() {
        let test_provider_name = "test_provider";
        temp_env::with_var("DEFAULT_PROVIDER", Some(test_provider_name), || {
            let s = match load_settings(Path::new("not_exist_file.toml")) {
                Ok(s) => s,
                Err(e) => panic!("{}", e),
            };
            assert_eq!(s.default_provider, test_provider_name);
        });
    }

    #[test]
    #[serial_test::serial]
    fn test_load_settings_with_dotenv() {
        let env_path = Path::new("tests/.env.test");
        fs::create_dir_all("tests").unwrap();
        let test_provider_name = "test_openweather";
        {
            let mut file = fs::File::create(env_path).unwrap();
            writeln!(file, "DEFAULT_PROVIDER={}", test_provider_name).unwrap();
        }
        // Dotenv interferes with other tests
        temp_env::with_var("DEFAULT_PROVIDER", None::<String>, || {
            dotenvy::from_filename(env_path).unwrap();

            let s = match load_settings(Path::new("not_exist_file.toml")) {
                Ok(s) => s,
                Err(e) => {
                    fs::remove_file(env_path).unwrap();
                    panic!("{}", e)
                },
            };

            assert_eq!(s.default_provider, test_provider_name);

            fs::remove_file(env_path).unwrap();
        });
    }

    #[test]
    #[serial_test::serial]
    fn test_load_settings_from_file() {
        let settings_path = Path::new("tests/settings_test.toml");
        fs::create_dir_all("tests").unwrap();
        let test_provider_name = "test_from_storage_provider";
        {
            let mut file = fs::File::create(settings_path).unwrap();
            writeln!(file, r#"default_provider = "{}""#, test_provider_name).unwrap();
        }
        let s = match load_settings(settings_path) {
            Ok(s) => s,
            Err(e) => {
                fs::remove_file(settings_path).unwrap();
                panic!("{}", e)
            },
        };
        assert_eq!(s.default_provider, test_provider_name);
        fs::remove_file(settings_path).unwrap();
    }
}
