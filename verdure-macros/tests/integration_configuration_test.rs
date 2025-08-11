use std::collections::HashMap;
use verdure::config::{ConfigComponent, ConfigManager, ConfigSource};
use verdure_macros::Configuration;

#[derive(Configuration, Debug)]
#[configuration("server")]
struct ServerConfig {
    #[config_default(8080)]
    port: Option<u16>,
    #[config_default("localhost")]
    host: Option<String>,
    name: Option<String>,
    #[config_default(true)]
    debug: Option<bool>,
}

#[test]
fn test_configuration_derive() {
    assert_eq!(ServerConfig::config_module_key(), "server")
}

#[test]
fn test_configuration_with_config_manager() {
    let mut config_manager = ConfigManager::new();

    let mut props = HashMap::new();
    props.insert("server.port".to_string(), "9000".to_string());
    props.insert("server.host".to_string(), "0.0.0.0".to_string());
    props.insert("server.debug".to_string(), "false".to_string());
    props.insert("server.name".to_string(), "TestApp".to_string());

    config_manager
        .add_source(ConfigSource::Properties(props))
        .unwrap();

    let config = ServerConfig::from_config_manager(&config_manager).unwrap();

    assert_eq!(config.port, Some(9000));
    assert_eq!(config.host, Some("0.0.0.0".to_string()));
    assert_eq!(config.debug, Some(false));
    assert_eq!(config.name, Some("TestApp".to_string()));
}

#[test]
fn test_configuration_with_defaults() {
    let config_manager = ConfigManager::new();
    let config = ServerConfig::from_config_manager(&config_manager).unwrap();

    assert_eq!(config.port, Some(8080));
    assert_eq!(config.host, Some("localhost".to_string()));
    assert_eq!(config.debug, Some(true));
    assert_eq!(config.name, None);
}