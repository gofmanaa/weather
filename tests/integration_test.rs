use assert_cmd::cargo;
use std::fs;
use std::path::PathBuf;

fn setup_test_config(path: &PathBuf) {
    let config_content = r#"default_provider = "weatherapi""#.to_string();
    fs::write(path, config_content).unwrap();
}

#[test]
fn success_set_default_provider() {
    let config_path = PathBuf::from("tests/test_settings.toml");
    setup_test_config(&config_path);

    let mut cmd = cargo::cargo_bin_cmd!();
    cmd.env("OPENWEATHER_API_KEY", "test_api_key")
        .arg("--config-path")
        .arg(&config_path)
        .arg("configure")
        .arg("openweather")
        .assert()
        .success()
        .stdout(predicates::str::contains(format!(
            "Default provider saved to {}",
            config_path.display()
        )));

    let settings = fs::read_to_string(&config_path).expect("Failed to read test_settings.toml");

    assert!(settings.contains(r#"default_provider = "openweather""#));

    fs::remove_file(config_path).unwrap();
}

#[test]
fn success_list_supported_providers() {
    let mut cmd = cargo::cargo_bin_cmd!();
    cmd.env("OPENWEATHER_API_KEY", "test_api_key")
        .env("WEATHERAPI_API_KEY", "test_api_key")
        .arg("configure")
        .assert()
        .stdout(predicates::str::contains("Available providers:"))
        .stdout(predicates::str::contains("openweather"))
        .stdout(predicates::str::contains("weatherapi"));
}

#[test]
fn fail_to_set_not_supported_provider() {
    let mut cmd = cargo::cargo_bin_cmd!();
    let not_supported_provider = "not_supported_provider";
    cmd.env("OPENWEATHER_API_KEY", "test_api_key")
        .arg("configure")
        .arg(not_supported_provider)
        .assert()
        .stderr(predicates::str::contains(format!(
            "Provider `{not_supported_provider}` not supported",
        )));
}
