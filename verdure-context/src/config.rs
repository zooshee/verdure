//! Configuration management system
//!
//! This module provides configuration management functionality for the Verdure context system.
//! It supports hierarchical configuration sources, property binding, type-safe configuration
//! access, and integration with environment profiles.

use crate::error::{ContextError, ContextResult};
use crate::profile::ProfileManager;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Configuration file formats
#[derive(Debug, Clone, Copy)]
enum ConfigFileFormat {
    Toml,
    Yaml,
    Properties,
}

/// Configuration source types
/// 
/// `ConfigSource` represents different sources from which configuration
/// can be loaded, supporting various formats and locations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigSource {
    /// Configuration from a TOML file
    TomlFile(String),
    /// Configuration from a YAML file
    YamlFile(String),
    /// Configuration from a Properties file
    PropertiesFile(String),
    /// Configuration from any file (auto-detect format)
    ConfigFile(String),
    /// Configuration from environment variables
    Environment,
    /// Configuration from command line arguments
    CommandLine,
    /// In-memory configuration properties
    Properties(HashMap<String, String>),
}

/// Configuration value types
/// 
/// `ConfigValue` represents different types of configuration values
/// that can be stored and retrieved from the configuration system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigValue {
    /// String value
    String(String),
    /// Integer value
    Integer(i64),
    /// Float value
    Float(f64),
    /// Boolean value
    Boolean(bool),
    /// Array of values
    Array(Vec<ConfigValue>),
    /// Nested object/map
    Object(HashMap<String, ConfigValue>),
}

impl ConfigValue {
    /// Converts the value to a string if possible
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use verdure_context::ConfigValue;
    /// 
    /// let value = ConfigValue::String("hello".to_string());
    /// assert_eq!(value.as_string(), Some("hello".to_string()));
    /// 
    /// let value = ConfigValue::Integer(42);
    /// assert_eq!(value.as_string(), Some("42".to_string()));
    /// ```
    pub fn as_string(&self) -> Option<String> {
        match self {
            ConfigValue::String(s) => Some(s.clone()),
            ConfigValue::Integer(i) => Some(i.to_string()),
            ConfigValue::Float(f) => Some(f.to_string()),
            ConfigValue::Boolean(b) => Some(b.to_string()),
            _ => None,
        }
    }
    
    /// Converts the value to an integer if possible
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use verdure_context::ConfigValue;
    /// 
    /// let value = ConfigValue::Integer(42);
    /// assert_eq!(value.as_integer(), Some(42));
    /// 
    /// let value = ConfigValue::String("123".to_string());
    /// assert_eq!(value.as_integer(), Some(123));
    /// ```
    pub fn as_integer(&self) -> Option<i64> {
        match self {
            ConfigValue::Integer(i) => Some(*i),
            ConfigValue::String(s) => s.parse().ok(),
            _ => None,
        }
    }
    
    /// Converts the value to a float if possible
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use verdure_context::ConfigValue;
    /// 
    /// let value = ConfigValue::Float(3.14);
    /// assert_eq!(value.as_float(), Some(3.14));
    /// 
    /// let value = ConfigValue::Integer(42);
    /// assert_eq!(value.as_float(), Some(42.0));
    /// ```
    pub fn as_float(&self) -> Option<f64> {
        match self {
            ConfigValue::Float(f) => Some(*f),
            ConfigValue::Integer(i) => Some(*i as f64),
            ConfigValue::String(s) => s.parse().ok(),
            _ => None,
        }
    }
    
    /// Converts the value to a boolean if possible
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use verdure_context::ConfigValue;
    /// 
    /// let value = ConfigValue::Boolean(true);
    /// assert_eq!(value.as_boolean(), Some(true));
    /// 
    /// let value = ConfigValue::String("true".to_string());
    /// assert_eq!(value.as_boolean(), Some(true));
    /// ```
    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            ConfigValue::Boolean(b) => Some(*b),
            ConfigValue::String(s) => {
                match s.to_lowercase().as_str() {
                    "true" | "yes" | "on" | "1" => Some(true),
                    "false" | "no" | "off" | "0" => Some(false),
                    _ => None,
                }
            }
            ConfigValue::Integer(i) => Some(*i != 0),
            _ => None,
        }
    }
    
    /// Converts the value to an array if possible
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use verdure_context::ConfigValue;
    /// 
    /// let value = ConfigValue::Array(vec![
    ///     ConfigValue::String("a".to_string()),
    ///     ConfigValue::String("b".to_string()),
    /// ]);
    /// assert!(value.as_array().is_some());
    /// ```
    pub fn as_array(&self) -> Option<&Vec<ConfigValue>> {
        match self {
            ConfigValue::Array(arr) => Some(arr),
            _ => None,
        }
    }
    
    /// Converts the value to an object/map if possible
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use verdure_context::ConfigValue;
    /// use std::collections::HashMap;
    /// 
    /// let mut obj = HashMap::new();
    /// obj.insert("key".to_string(), ConfigValue::String("value".to_string()));
    /// 
    /// let value = ConfigValue::Object(obj);
    /// assert!(value.as_object().is_some());
    /// ```
    pub fn as_object(&self) -> Option<&HashMap<String, ConfigValue>> {
        match self {
            ConfigValue::Object(obj) => Some(obj),
            _ => None,
        }
    }
}

/// Configuration manager
/// 
/// `ConfigManager` provides comprehensive configuration management functionality,
/// including loading from multiple sources, hierarchical property resolution,
/// type-safe access, and integration with environment profiles.
/// 
/// # Examples
/// 
/// ```rust
/// use verdure_context::{ConfigManager, ConfigSource};
/// use std::collections::HashMap;
/// 
/// let mut manager = ConfigManager::new();
/// 
/// // Add configuration from properties
/// let mut props = HashMap::new();
/// props.insert("app.name".to_string(), "MyApp".to_string());
/// props.insert("app.port".to_string(), "8080".to_string());
/// 
/// manager.add_source(ConfigSource::Properties(props)).unwrap();
/// 
/// assert_eq!(manager.get_string("app.name").unwrap(), "MyApp");
/// assert_eq!(manager.get_integer("app.port").unwrap(), 8080);
/// ```
#[derive(Debug)]
pub struct ConfigManager {
    /// Configuration sources in order of precedence (last added has highest precedence)
    sources: Vec<ConfigSource>,
    /// Cached configuration values
    cache: DashMap<String, ConfigValue>,
    /// Profile manager for environment-specific configurations
    profile_manager: ProfileManager,
}

impl ConfigManager {
    /// Creates a new configuration manager
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use verdure_context::ConfigManager;
    /// 
    /// let manager = ConfigManager::new();
    /// ```
    pub fn new() -> Self {
        Self {
            sources: Vec::new(),
            cache: DashMap::new(),
            profile_manager: ProfileManager::new(),
        }
    }
    
    /// Adds a configuration source
    /// 
    /// Sources added later have higher precedence than those added earlier.
    /// 
    /// # Arguments
    /// 
    /// * `source` - The configuration source to add
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use verdure_context::{ConfigManager, ConfigSource};
    /// use std::collections::HashMap;
    /// 
    /// let mut manager = ConfigManager::new();
    /// let props = HashMap::new();
    /// 
    /// manager.add_source(ConfigSource::Properties(props)).unwrap();
    /// ```
    pub fn add_source(&mut self, source: ConfigSource) -> ContextResult<()> {
        self.sources.push(source);
        self.invalidate_cache();
        Ok(())
    }
    
    /// Loads configuration from a TOML file
    /// 
    /// # Arguments
    /// 
    /// * `path` - Path to the TOML configuration file
    /// 
    /// # Examples
    /// 
    /// ```rust,no_run
    /// use verdure_context::ConfigManager;
    /// 
    /// let mut manager = ConfigManager::new();
    /// manager.load_from_toml_file("config/app.toml").unwrap();
    /// ```
    pub fn load_from_toml_file<P: AsRef<Path>>(&mut self, path: P) -> ContextResult<()> {
        let path_str = path.as_ref().to_string_lossy().to_string();
        self.add_source(ConfigSource::TomlFile(path_str))
    }
    
    /// Loads configuration from a YAML file
    /// 
    /// # Arguments
    /// 
    /// * `path` - Path to the YAML configuration file
    /// 
    /// # Examples
    /// 
    /// ```rust,no_run
    /// use verdure_context::ConfigManager;
    /// 
    /// let mut manager = ConfigManager::new();
    /// manager.load_from_yaml_file("config/app.yaml").unwrap();
    /// ```
    pub fn load_from_yaml_file<P: AsRef<Path>>(&mut self, path: P) -> ContextResult<()> {
        let path_str = path.as_ref().to_string_lossy().to_string();
        self.add_source(ConfigSource::YamlFile(path_str))
    }
    
    /// Loads configuration from a Properties file
    /// 
    /// # Arguments
    /// 
    /// * `path` - Path to the Properties configuration file
    /// 
    /// # Examples
    /// 
    /// ```rust,no_run
    /// use verdure_context::ConfigManager;
    /// 
    /// let mut manager = ConfigManager::new();
    /// manager.load_from_properties_file("config/app.properties").unwrap();
    /// ```
    pub fn load_from_properties_file<P: AsRef<Path>>(&mut self, path: P) -> ContextResult<()> {
        let path_str = path.as_ref().to_string_lossy().to_string();
        self.add_source(ConfigSource::PropertiesFile(path_str))
    }
    
    /// Loads configuration from a file with automatic format detection
    /// 
    /// The format is detected based on the file extension:
    /// - `.toml` -> TOML format
    /// - `.yaml` or `.yml` -> YAML format  
    /// - `.properties` -> Properties format
    /// - Others -> Attempts to parse as TOML first, then YAML, then Properties
    /// 
    /// # Arguments
    /// 
    /// * `path` - Path to the configuration file
    /// 
    /// # Examples
    /// 
    /// ```rust,no_run
    /// use verdure_context::ConfigManager;
    /// 
    /// let mut manager = ConfigManager::new();
    /// manager.load_from_config_file("config/app.yaml").unwrap();
    /// manager.load_from_config_file("config/database.properties").unwrap();
    /// manager.load_from_config_file("config/server.toml").unwrap();
    /// ```
    pub fn load_from_config_file<P: AsRef<Path>>(&mut self, path: P) -> ContextResult<()> {
        let path_str = path.as_ref().to_string_lossy().to_string();
        self.add_source(ConfigSource::ConfigFile(path_str))
    }
    
    /// Gets a configuration value by key
    /// 
    /// # Arguments
    /// 
    /// * `key` - The configuration key (e.g., "database.url")
    /// 
    /// # Returns
    /// 
    /// The configuration value if found, `None` otherwise
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use verdure_context::{ConfigManager, ConfigSource};
    /// use std::collections::HashMap;
    /// 
    /// let mut manager = ConfigManager::new();
    /// let mut props = HashMap::new();
    /// props.insert("test.key".to_string(), "test.value".to_string());
    /// 
    /// manager.add_source(ConfigSource::Properties(props)).unwrap();
    /// 
    /// let value = manager.get("test.key");
    /// assert!(value.is_some());
    /// ```
    pub fn get(&self, key: &str) -> Option<ConfigValue> {
        // Check cache first
        if let Some(cached) = self.cache.get(key) {
            return Some(cached.clone());
        }
        
        // Check profile manager first (profiles have highest precedence)
        if let Some(profile_value) = self.profile_manager.get_property(key) {
            let value = ConfigValue::String(profile_value.to_string());
            self.cache.insert(key.to_string(), value.clone());
            return Some(value);
        }
        
        // Check sources in reverse order (last added has highest precedence)
        for source in self.sources.iter().rev() {
            if let Some(value) = self.get_from_source(source, key) {
                self.cache.insert(key.to_string(), value.clone());
                return Some(value);
            }
        }
        
        None
    }
    
    /// Gets a configuration value as a string
    /// 
    /// # Arguments
    /// 
    /// * `key` - The configuration key
    /// 
    /// # Returns
    /// 
    /// The configuration value as a string
    /// 
    /// # Errors
    /// 
    /// Returns an error if the key is not found or cannot be converted to a string
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use verdure_context::{ConfigManager, ConfigSource};
    /// use std::collections::HashMap;
    /// 
    /// let mut manager = ConfigManager::new();
    /// let mut props = HashMap::new();
    /// props.insert("app.name".to_string(), "MyApp".to_string());
    /// 
    /// manager.add_source(ConfigSource::Properties(props)).unwrap();
    /// 
    /// assert_eq!(manager.get_string("app.name").unwrap(), "MyApp");
    /// ```
    pub fn get_string(&self, key: &str) -> ContextResult<String> {
        self.get(key)
            .and_then(|v| v.as_string())
            .ok_or_else(|| ContextError::configuration_not_found(key))
    }
    
    /// Gets a configuration value as an integer
    /// 
    /// # Arguments
    /// 
    /// * `key` - The configuration key
    /// 
    /// # Returns
    /// 
    /// The configuration value as an integer
    /// 
    /// # Errors
    /// 
    /// Returns an error if the key is not found or cannot be converted to an integer
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use verdure_context::{ConfigManager, ConfigSource};
    /// use std::collections::HashMap;
    /// 
    /// let mut manager = ConfigManager::new();
    /// let mut props = HashMap::new();
    /// props.insert("app.port".to_string(), "8080".to_string());
    /// 
    /// manager.add_source(ConfigSource::Properties(props)).unwrap();
    /// 
    /// assert_eq!(manager.get_integer("app.port").unwrap(), 8080);
    /// ```
    pub fn get_integer(&self, key: &str) -> ContextResult<i64> {
        self.get(key)
            .and_then(|v| v.as_integer())
            .ok_or_else(|| ContextError::configuration_not_found(key))
    }
    
    /// Gets a configuration value as a float
    /// 
    /// # Arguments
    /// 
    /// * `key` - The configuration key
    /// 
    /// # Returns
    /// 
    /// The configuration value as a float
    /// 
    /// # Errors
    /// 
    /// Returns an error if the key is not found or cannot be converted to a float
    pub fn get_float(&self, key: &str) -> ContextResult<f64> {
        self.get(key)
            .and_then(|v| v.as_float())
            .ok_or_else(|| ContextError::configuration_not_found(key))
    }
    
    /// Gets a configuration value as a boolean
    /// 
    /// # Arguments
    /// 
    /// * `key` - The configuration key
    /// 
    /// # Returns
    /// 
    /// The configuration value as a boolean
    /// 
    /// # Errors
    /// 
    /// Returns an error if the key is not found or cannot be converted to a boolean
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use verdure_context::{ConfigManager, ConfigSource};
    /// use std::collections::HashMap;
    /// 
    /// let mut manager = ConfigManager::new();
    /// let mut props = HashMap::new();
    /// props.insert("app.debug".to_string(), "true".to_string());
    /// 
    /// manager.add_source(ConfigSource::Properties(props)).unwrap();
    /// 
    /// assert_eq!(manager.get_boolean("app.debug").unwrap(), true);
    /// ```
    pub fn get_boolean(&self, key: &str) -> ContextResult<bool> {
        self.get(key)
            .and_then(|v| v.as_boolean())
            .ok_or_else(|| ContextError::configuration_not_found(key))
    }
    
    /// Gets a configuration value with a default fallback
    /// 
    /// # Arguments
    /// 
    /// * `key` - The configuration key
    /// * `default` - The default value to return if key is not found
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use verdure_context::ConfigManager;
    /// 
    /// let manager = ConfigManager::new();
    /// 
    /// assert_eq!(manager.get_string_or_default("missing.key", "default"), "default");
    /// ```
    pub fn get_string_or_default(&self, key: &str, default: &str) -> String {
        self.get_string(key).unwrap_or_else(|_| default.to_string())
    }
    
    /// Gets an integer configuration value with a default fallback
    /// 
    /// # Arguments
    /// 
    /// * `key` - The configuration key
    /// * `default` - The default value to return if key is not found
    pub fn get_integer_or_default(&self, key: &str, default: i64) -> i64 {
        self.get_integer(key).unwrap_or(default)
    }
    
    /// Gets a boolean configuration value with a default fallback
    /// 
    /// # Arguments
    /// 
    /// * `key` - The configuration key
    /// * `default` - The default value to return if key is not found
    pub fn get_boolean_or_default(&self, key: &str, default: bool) -> bool {
        self.get_boolean(key).unwrap_or(default)
    }
    
    /// Gets a reference to the profile manager
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use verdure_context::ConfigManager;
    /// 
    /// let manager = ConfigManager::new();
    /// let profile_manager = manager.profile_manager();
    /// ```
    pub fn profile_manager(&self) -> &ProfileManager {
        &self.profile_manager
    }
    
    /// Gets a mutable reference to the profile manager
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use verdure_context::{ConfigManager, Profile};
    /// use std::collections::HashMap;
    /// 
    /// let mut manager = ConfigManager::new();
    /// let profile = Profile::new("development", HashMap::new());
    /// 
    /// manager.profile_manager_mut().add_profile(profile).unwrap();
    /// ```
    pub fn profile_manager_mut(&mut self) -> &mut ProfileManager {
        &mut self.profile_manager
    }
    
    /// Sets a configuration property
    /// 
    /// # Arguments
    /// 
    /// * `key` - The configuration key
    /// * `value` - The configuration value
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use verdure_context::{ConfigManager, ConfigValue};
    /// 
    /// let mut manager = ConfigManager::new();
    /// manager.set("runtime.property", ConfigValue::String("value".to_string()));
    /// 
    /// assert_eq!(manager.get_string("runtime.property").unwrap(), "value");
    /// ```
    pub fn set(&mut self, key: &str, value: ConfigValue) {
        self.cache.insert(key.to_string(), value);
    }
    
    /// Gets the number of configuration sources
    /// 
    /// # Returns
    /// 
    /// The total number of configuration sources
    pub fn sources_count(&self) -> usize {
        self.sources.len()
    }
    
    /// Invalidates the configuration cache
    /// 
    /// This forces the manager to reload configuration values from sources
    /// on the next access.
    pub fn invalidate_cache(&self) {
        self.cache.clear();
    }
    
    // Helper method to get value from a specific source
    fn get_from_source(&self, source: &ConfigSource, key: &str) -> Option<ConfigValue> {
        match source {
            ConfigSource::Properties(props) => {
                props.get(key).map(|v| ConfigValue::String(v.clone()))
            }
            ConfigSource::Environment => {
                // Convert key to environment variable format (e.g., "app.port" -> "APP_PORT")
                let env_key = key.to_uppercase().replace('.', "_");
                std::env::var(&env_key)
                    .ok()
                    .map(|v| ConfigValue::String(v))
            }
            ConfigSource::TomlFile(path) => {
                self.load_file_config(path, ConfigFileFormat::Toml)
                    .and_then(|props| props.get(key).map(|v| ConfigValue::String(v.clone())))
            }
            ConfigSource::YamlFile(path) => {
                self.load_file_config(path, ConfigFileFormat::Yaml)
                    .and_then(|props| props.get(key).map(|v| ConfigValue::String(v.clone())))
            }
            ConfigSource::PropertiesFile(path) => {
                self.load_file_config(path, ConfigFileFormat::Properties)
                    .and_then(|props| props.get(key).map(|v| ConfigValue::String(v.clone())))
            }
            ConfigSource::ConfigFile(path) => {
                self.load_file_config_auto_detect(path)
                    .and_then(|props| props.get(key).map(|v| ConfigValue::String(v.clone())))
            }
            _ => None, // TODO: Implement other source types
        }
    }
    
    // Helper method to load configuration from file
    fn load_file_config(&self, path: &str, format: ConfigFileFormat) -> Option<HashMap<String, String>> {
        let content = std::fs::read_to_string(path).ok()?;
        
        match format {
            ConfigFileFormat::Toml => {
                let toml_value: toml::Value = toml::from_str(&content).ok()?;
                self.toml_value_to_config_map(&toml_value, "").ok()
            }
            ConfigFileFormat::Yaml => {
                let yaml_value: serde_yaml::Value = serde_yaml::from_str(&content).ok()?;
                self.yaml_value_to_config_map(&yaml_value, "").ok()
            }
            ConfigFileFormat::Properties => {
                self.parse_properties(&content).ok()
            }
        }
    }
    
    // Helper method to auto-detect file format and load configuration
    fn load_file_config_auto_detect(&self, path: &str) -> Option<HashMap<String, String>> {
        let path_lower = path.to_lowercase();
        
        // Try to detect format by extension first
        if path_lower.ends_with(".toml") {
            return self.load_file_config(path, ConfigFileFormat::Toml);
        } else if path_lower.ends_with(".yaml") || path_lower.ends_with(".yml") {
            return self.load_file_config(path, ConfigFileFormat::Yaml);
        } else if path_lower.ends_with(".properties") {
            return self.load_file_config(path, ConfigFileFormat::Properties);
        }
        
        // If extension doesn't match known formats, try parsing in order: TOML, YAML, Properties
        if let Some(config) = self.load_file_config(path, ConfigFileFormat::Toml) {
            return Some(config);
        }
        
        if let Some(config) = self.load_file_config(path, ConfigFileFormat::Yaml) {
            return Some(config);
        }
        
        self.load_file_config(path, ConfigFileFormat::Properties)
    }
    
    // Helper method to convert YAML value to flat configuration map
    fn yaml_value_to_config_map(&self, value: &serde_yaml::Value, prefix: &str) -> ContextResult<HashMap<String, String>> {
        let mut map = HashMap::new();
        
        match value {
            serde_yaml::Value::Mapping(mapping) => {
                for (key, val) in mapping {
                    if let Some(key_str) = key.as_str() {
                        let full_key = if prefix.is_empty() {
                            key_str.to_string()
                        } else {
                            format!("{}.{}", prefix, key_str)
                        };
                        
                        match val {
                            serde_yaml::Value::Mapping(_) => {
                                // Recursively process nested mappings
                                let nested_map = self.yaml_value_to_config_map(val, &full_key)?;
                                map.extend(nested_map);
                            }
                            _ => {
                                // Convert primitive values to strings
                                map.insert(full_key, self.yaml_value_to_string(val));
                            }
                        }
                    }
                }
            }
            _ => {
                // For non-mapping values, use the prefix as the key
                if !prefix.is_empty() {
                    map.insert(prefix.to_string(), self.yaml_value_to_string(value));
                }
            }
        }
        
        Ok(map)
    }
    
    // Helper method to convert YAML value to string
    fn yaml_value_to_string(&self, value: &serde_yaml::Value) -> String {
        match value {
            serde_yaml::Value::String(s) => s.clone(),
            serde_yaml::Value::Number(n) => n.to_string(),
            serde_yaml::Value::Bool(b) => b.to_string(),
            serde_yaml::Value::Sequence(arr) => {
                // Convert array to comma-separated string
                arr.iter()
                    .map(|v| self.yaml_value_to_string(v))
                    .collect::<Vec<_>>()
                    .join(",")
            }
            serde_yaml::Value::Null => "".to_string(),
            _ => format!("{:?}", value),
        }
    }
    
    // Helper method to parse Properties format
    fn parse_properties(&self, content: &str) -> ContextResult<HashMap<String, String>> {
        let mut map = HashMap::new();
        
        for line in content.lines() {
            let line = line.trim();
            
            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') || line.starts_with('!') {
                continue;
            }
            
            // Find the first '=' or ':' separator
            if let Some(separator_pos) = line.find('=').or_else(|| line.find(':')) {
                let key = line[..separator_pos].trim().to_string();
                let value = line[separator_pos + 1..].trim().to_string();
                
                // Handle escaped characters and line continuations
                let processed_value = self.process_properties_value(&value);
                
                map.insert(key, processed_value);
            }
        }
        
        Ok(map)
    }
    
    // Helper method to process Properties file values (handle escaping, etc.)
    fn process_properties_value(&self, value: &str) -> String {
        // Basic processing - handle common escape sequences
        value
            .replace("\\n", "\n")
            .replace("\\t", "\t")
            .replace("\\r", "\r")
            .replace("\\\\", "\\")
    }
    fn toml_value_to_config_map(&self, value: &toml::Value, prefix: &str) -> ContextResult<HashMap<String, String>> {
        let mut map = HashMap::new();
        
        match value {
            toml::Value::Table(table) => {
                for (key, val) in table {
                    let full_key = if prefix.is_empty() {
                        key.clone()
                    } else {
                        format!("{}.{}", prefix, key)
                    };
                    
                    match val {
                        toml::Value::Table(_) => {
                            // Recursively process nested tables
                            let nested_map = self.toml_value_to_config_map(val, &full_key)?;
                            map.extend(nested_map);
                        }
                        _ => {
                            // Convert primitive values to strings
                            map.insert(full_key, self.toml_value_to_string(val));
                        }
                    }
                }
            }
            _ => {
                // For non-table values, use the prefix as the key
                if !prefix.is_empty() {
                    map.insert(prefix.to_string(), self.toml_value_to_string(value));
                }
            }
        }
        
        Ok(map)
    }
    
    // Helper method to convert TOML value to string
    fn toml_value_to_string(&self, value: &toml::Value) -> String {
        match value {
            toml::Value::String(s) => s.clone(),
            toml::Value::Integer(i) => i.to_string(),
            toml::Value::Float(f) => f.to_string(),
            toml::Value::Boolean(b) => b.to_string(),
            toml::Value::Array(arr) => {
                // Convert array to comma-separated string
                arr.iter()
                    .map(|v| self.toml_value_to_string(v))
                    .collect::<Vec<_>>()
                    .join(",")
            }
            _ => value.to_string(),
        }
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_value_conversions() {
        // String conversion
        let value = ConfigValue::String("hello".to_string());
        assert_eq!(value.as_string(), Some("hello".to_string()));
        
        // Integer conversion
        let value = ConfigValue::Integer(42);
        assert_eq!(value.as_integer(), Some(42));
        assert_eq!(value.as_string(), Some("42".to_string()));
        assert_eq!(value.as_float(), Some(42.0));
        
        // Boolean conversion
        let value = ConfigValue::Boolean(true);
        assert_eq!(value.as_boolean(), Some(true));
        assert_eq!(value.as_string(), Some("true".to_string()));
        
        // String to boolean conversion
        let value = ConfigValue::String("yes".to_string());
        assert_eq!(value.as_boolean(), Some(true));
        
        let value = ConfigValue::String("false".to_string());
        assert_eq!(value.as_boolean(), Some(false));
        
        // String to integer conversion
        let value = ConfigValue::String("123".to_string());
        assert_eq!(value.as_integer(), Some(123));
        
        // Invalid conversions
        let value = ConfigValue::String("not_a_number".to_string());
        assert_eq!(value.as_integer(), None);
        
        let value = ConfigValue::Array(vec![]);
        assert_eq!(value.as_string(), None);
    }

    #[test]
    fn test_config_manager_creation() {
        let manager = ConfigManager::new();
        assert!(manager.sources.is_empty());
    }

    #[test]
    fn test_config_manager_properties_source() {
        let mut manager = ConfigManager::new();
        let mut props = HashMap::new();
        props.insert("app.name".to_string(), "TestApp".to_string());
        props.insert("app.port".to_string(), "8080".to_string());
        props.insert("app.debug".to_string(), "true".to_string());
        
        manager.add_source(ConfigSource::Properties(props)).unwrap();
        
        assert_eq!(manager.get_string("app.name").unwrap(), "TestApp");
        assert_eq!(manager.get_integer("app.port").unwrap(), 8080);
        assert_eq!(manager.get_boolean("app.debug").unwrap(), true);
        
        // Test missing key
        assert!(manager.get_string("missing.key").is_err());
    }

    #[test]
    fn test_config_manager_defaults() {
        let manager = ConfigManager::new();
        
        assert_eq!(manager.get_string_or_default("missing.key", "default"), "default");
        assert_eq!(manager.get_integer_or_default("missing.key", 42), 42);
        assert_eq!(manager.get_boolean_or_default("missing.key", true), true);
    }

    #[test]
    fn test_config_manager_source_precedence() {
        let mut manager = ConfigManager::new();
        
        // Add first source
        let mut props1 = HashMap::new();
        props1.insert("app.name".to_string(), "App1".to_string());
        props1.insert("app.version".to_string(), "1.0".to_string());
        manager.add_source(ConfigSource::Properties(props1)).unwrap();
        
        // Add second source with overlapping key
        let mut props2 = HashMap::new();
        props2.insert("app.name".to_string(), "App2".to_string());
        props2.insert("app.port".to_string(), "8080".to_string());
        manager.add_source(ConfigSource::Properties(props2)).unwrap();
        
        // Second source should take precedence
        assert_eq!(manager.get_string("app.name").unwrap(), "App2");
        assert_eq!(manager.get_string("app.version").unwrap(), "1.0"); // Only in first source
        assert_eq!(manager.get_string("app.port").unwrap(), "8080"); // Only in second source
    }

    #[test]
    fn test_config_manager_cache() {
        let mut manager = ConfigManager::new();
        let mut props = HashMap::new();
        props.insert("test.key".to_string(), "test.value".to_string());
        
        manager.add_source(ConfigSource::Properties(props)).unwrap();
        
        // First access should populate cache
        assert_eq!(manager.get_string("test.key").unwrap(), "test.value");
        
        // Second access should use cache
        assert_eq!(manager.get_string("test.key").unwrap(), "test.value");
        
        // Manual cache update
        manager.set("runtime.key", ConfigValue::String("runtime.value".to_string()));
        assert_eq!(manager.get_string("runtime.key").unwrap(), "runtime.value");
    }

    #[test]
    fn test_config_manager_cache_invalidation() {
        let mut manager = ConfigManager::new();
        let mut props = HashMap::new();
        props.insert("test.key".to_string(), "test.value".to_string());
        
        manager.add_source(ConfigSource::Properties(props)).unwrap();
        
        // Access to populate cache
        assert_eq!(manager.get_string("test.key").unwrap(), "test.value");
        
        // Invalidate cache
        manager.invalidate_cache();
        
        // Should still work (reloaded from source)
        assert_eq!(manager.get_string("test.key").unwrap(), "test.value");
    }

    #[test]
    fn test_yaml_parsing() {
        let yaml_content = r#"
app:
  name: "TestApp"
  port: 8080
  features:
    - "auth"
    - "logging"
database:
  host: "localhost"
  port: 5432
  ssl: true
"#;
        
        let manager = ConfigManager::new();
        let yaml_value: serde_yaml::Value = serde_yaml::from_str(yaml_content).unwrap();
        let config_map = manager.yaml_value_to_config_map(&yaml_value, "").unwrap();
        
        assert_eq!(config_map.get("app.name"), Some(&"TestApp".to_string()));
        assert_eq!(config_map.get("app.port"), Some(&"8080".to_string()));
        assert_eq!(config_map.get("database.host"), Some(&"localhost".to_string()));
        assert_eq!(config_map.get("database.ssl"), Some(&"true".to_string()));
        assert_eq!(config_map.get("app.features"), Some(&"auth,logging".to_string()));
    }

    #[test]
    fn test_properties_parsing() {
        let properties_content = r#"
# Application configuration
app.name=TestApp
app.port=8080
app.debug=true

# Database configuration
database.host=localhost
database.port=5432
database.ssl=true

# Comments and empty lines should be ignored
! This is another comment style
"#;
        
        let manager = ConfigManager::new();
        let config_map = manager.parse_properties(properties_content).unwrap();
        
        assert_eq!(config_map.get("app.name"), Some(&"TestApp".to_string()));
        assert_eq!(config_map.get("app.port"), Some(&"8080".to_string()));
        assert_eq!(config_map.get("app.debug"), Some(&"true".to_string()));
        assert_eq!(config_map.get("database.host"), Some(&"localhost".to_string()));
        assert_eq!(config_map.get("database.port"), Some(&"5432".to_string()));
        assert_eq!(config_map.get("database.ssl"), Some(&"true".to_string()));
    }

    #[test]
    fn test_properties_escape_sequences() {
        let properties_content = r#"
message.welcome=Hello\nWorld\tTest
file.path=C:\\Users\\Test
"#;
        
        let manager = ConfigManager::new();
        let config_map = manager.parse_properties(properties_content).unwrap();
        
        assert_eq!(config_map.get("message.welcome"), Some(&"Hello\nWorld\tTest".to_string()));
        assert_eq!(config_map.get("file.path"), Some(&"C:\\Users\\Test".to_string()));
    }

    #[test]
    fn test_config_source_types() {
        let mut manager = ConfigManager::new();
        
        // Test different source types
        let mut props = HashMap::new();
        props.insert("source.type".to_string(), "properties".to_string());
        
        manager.add_source(ConfigSource::Properties(props)).unwrap();
        
        assert_eq!(manager.get_string("source.type").unwrap(), "properties");
    }

    #[test] 
    fn test_multiple_config_formats() {
        let mut manager = ConfigManager::new();
        
        // Add properties source
        let mut props = HashMap::new();
        props.insert("app.name".to_string(), "PropsApp".to_string());
        props.insert("app.version".to_string(), "1.0".to_string());
        manager.add_source(ConfigSource::Properties(props)).unwrap();
        
        // Add higher precedence properties (should override)
        let mut override_props = HashMap::new();
        override_props.insert("app.name".to_string(), "OverrideApp".to_string());
        override_props.insert("app.env".to_string(), "test".to_string());
        manager.add_source(ConfigSource::Properties(override_props)).unwrap();
        
        // Higher precedence source should win
        assert_eq!(manager.get_string("app.name").unwrap(), "OverrideApp");
        assert_eq!(manager.get_string("app.version").unwrap(), "1.0"); // Only in first source
        assert_eq!(manager.get_string("app.env").unwrap(), "test"); // Only in second source
    }
}