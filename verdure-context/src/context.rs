//! Application context implementation
//!
//! This module provides the main `ApplicationContext` implementation that serves as the
//! central hub for application-wide state, configuration, environment management, and
//! integration with the IoC container.

use crate::config::{ConfigFactory, ConfigManager, ConfigSource, ConfigValue};
use crate::error::{ContextError, ContextResult};
use crate::event::{
    ConfigurationChangedEvent, ContextAwareEventListener, ContextInitializedEvent,
    ContextInitializingEvent, Event, EventListener, EventPublisher, ProfileActivatedEvent,
};
use crate::profile::Profile;
use dashmap::DashMap;
use std::path::Path;
use std::sync::Arc;
use verdure_ioc::{ComponentContainer, ComponentFactory};

/// Application context builder
///
/// `ApplicationContextBuilder` provides a fluent API for constructing
/// an `ApplicationContext` with various configuration sources, profiles,
/// and settings.
///
/// # Examples
///
/// ```rust
/// use verdure_context::{ApplicationContextBuilder, Profile};
/// use std::collections::HashMap;
///
/// // Create the profile first
/// let profile = Profile::new("development", HashMap::new());
///
/// let context = ApplicationContextBuilder::new()
///     .with_profile(profile)
///     .with_active_profile("development")
///     .build()
///     .unwrap();
/// ```
#[derive(Debug)]
pub struct ApplicationContextBuilder {
    config_sources: Vec<ConfigSource>,
    active_profiles: Vec<String>,
    profiles: Vec<Profile>,
    properties: std::collections::HashMap<String, String>,
}

impl ApplicationContextBuilder {
    /// Creates a new application context builder
    ///
    /// # Examples
    ///
    /// ```rust
    /// use verdure_context::ApplicationContextBuilder;
    ///
    /// let builder = ApplicationContextBuilder::new();
    /// ```
    pub fn new() -> Self {
        Self {
            config_sources: Vec::new(),
            active_profiles: Vec::new(),
            profiles: Vec::new(),
            properties: std::collections::HashMap::new(),
        }
    }

    /// Adds a configuration source
    ///
    /// # Arguments
    ///
    /// * `source` - The configuration source to add
    ///
    /// # Examples
    ///
    /// ```rust
    /// use verdure_context::{ApplicationContextBuilder, ConfigSource};
    /// use std::collections::HashMap;
    ///
    /// let builder = ApplicationContextBuilder::new()
    ///     .with_config_source(ConfigSource::Environment);
    /// ```
    pub fn with_config_source(mut self, source: ConfigSource) -> Self {
        self.config_sources.push(source);
        self
    }

    /// Loads configuration from a TOML file
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the TOML configuration file
    ///
    /// # Examples
    ///
    /// ```rust
    /// use verdure_context::ApplicationContextBuilder;
    ///
    /// let builder = ApplicationContextBuilder::new()
    ///     .with_toml_config_file("config/app.toml");
    /// ```
    pub fn with_toml_config_file<P: AsRef<Path>>(mut self, path: P) -> Self {
        let path_str = path.as_ref().to_string_lossy().to_string();
        self.config_sources.push(ConfigSource::TomlFile(path_str));
        self
    }

    /// Loads configuration from a YAML file
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the YAML configuration file
    ///
    /// # Examples
    ///
    /// ```rust
    /// use verdure_context::ApplicationContextBuilder;
    ///
    /// let builder = ApplicationContextBuilder::new()
    ///     .with_yaml_config_file("config/app.yaml");
    /// ```
    pub fn with_yaml_config_file<P: AsRef<Path>>(mut self, path: P) -> Self {
        let path_str = path.as_ref().to_string_lossy().to_string();
        self.config_sources.push(ConfigSource::YamlFile(path_str));
        self
    }

    /// Loads configuration from a Properties file
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the Properties configuration file
    ///
    /// # Examples
    ///
    /// ```rust
    /// use verdure_context::ApplicationContextBuilder;
    ///
    /// let builder = ApplicationContextBuilder::new()
    ///     .with_properties_config_file("config/app.properties");
    /// ```
    pub fn with_properties_config_file<P: AsRef<Path>>(mut self, path: P) -> Self {
        let path_str = path.as_ref().to_string_lossy().to_string();
        self.config_sources
            .push(ConfigSource::PropertiesFile(path_str));
        self
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
    /// ```rust
    /// use verdure_context::ApplicationContextBuilder;
    ///
    /// let builder = ApplicationContextBuilder::new()
    ///     .with_config_file("config/app.yaml")
    ///     .with_config_file("config/database.properties")
    ///     .with_config_file("config/server.toml");
    /// ```
    pub fn with_config_file<P: AsRef<Path>>(mut self, path: P) -> Self {
        let path_str = path.as_ref().to_string_lossy().to_string();
        self.config_sources.push(ConfigSource::ConfigFile(path_str));
        self
    }

    /// Sets an active profile
    ///
    /// # Arguments
    ///
    /// * `profile` - The profile name to activate
    ///
    /// # Examples
    ///
    /// ```rust
    /// use verdure_context::ApplicationContextBuilder;
    ///
    /// let builder = ApplicationContextBuilder::new()
    ///     .with_active_profile("production");
    /// ```
    pub fn with_active_profile(mut self, profile: impl Into<String>) -> Self {
        self.active_profiles.push(profile.into());
        self
    }

    /// Adds a profile
    ///
    /// # Arguments
    ///
    /// * `profile` - The profile to add
    ///
    /// # Examples
    ///
    /// ```rust
    /// use verdure_context::{ApplicationContextBuilder, Profile};
    /// use std::collections::HashMap;
    ///
    /// let profile = Profile::new("test", HashMap::new());
    /// let builder = ApplicationContextBuilder::new()
    ///     .with_profile(profile);
    /// ```
    pub fn with_profile(mut self, profile: Profile) -> Self {
        self.profiles.push(profile);
        self
    }

    /// Sets a property value
    ///
    /// # Arguments
    ///
    /// * `key` - The property key
    /// * `value` - The property value
    ///
    /// # Examples
    ///
    /// ```rust
    /// use verdure_context::ApplicationContextBuilder;
    ///
    /// let builder = ApplicationContextBuilder::new()
    ///     .with_property("app.name", "MyApplication");
    /// ```
    pub fn with_property(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.properties.insert(key.into(), value.into());
        self
    }

    /// Builds the application context
    ///
    /// # Returns
    ///
    /// A new `ApplicationContext` instance configured with the specified settings
    ///
    /// # Errors
    ///
    /// Returns an error if context initialization fails
    ///
    /// # Examples
    ///
    /// ```rust
    /// use verdure_context::{ApplicationContextBuilder, Profile};
    /// use std::collections::HashMap;
    ///
    /// // Create the profile first
    /// let profile = Profile::new("development", HashMap::new());
    ///
    /// let context = ApplicationContextBuilder::new()
    ///     .with_profile(profile)
    ///     .with_active_profile("development")
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn build(self) -> ContextResult<ApplicationContext> {
        let context = ApplicationContext::new();

        // Add configuration sources
        for source in self.config_sources {
            context.config_manager.add_source(source)?;
        }

        // Add properties as a configuration source
        if !self.properties.is_empty() {
            context
                .config_manager
                .add_source(ConfigSource::Properties(self.properties))?;
        }

        // Add profiles
        for profile in self.profiles {
            context
                .config_manager
                .profile_manager_mut()
                .add_profile(profile)?;
        }

        // Activate profiles
        for profile_name in self.active_profiles {
            context
                .config_manager
                .profile_manager_mut()
                .activate_profile(&profile_name)?;

            // Get properties count for the activated profile
            let properties_count = context
                .config_manager
                .profile_manager()
                .get_profile_properties_count(&profile_name);

            // Publish profile activated event
            let event = ProfileActivatedEvent {
                profile_name: profile_name.clone(),
                properties_count,
                timestamp: std::time::SystemTime::now(),
            };
            context.event_publisher.publish(&event);
        }

        Ok(context)
    }
}

impl Default for ApplicationContextBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Main application context
///
/// `ApplicationContext` serves as the central hub for application-wide services,
/// configuration, environment management, and integration with the IoC container.
/// It provides a unified interface for accessing all framework services.
///
/// # Examples
///
/// ```rust
/// use verdure_context::ApplicationContext;
///
/// let context = ApplicationContext::builder()
///     .with_property("app.name", "MyApp")
///     .build()
///     .unwrap();
///
/// // Access configuration
/// let app_name = context.get_config("app.name");
///
/// // Access IoC container
/// let container = context.container();
/// ```
pub struct ApplicationContext {
    /// Configuration manager
    config_manager: Arc<ConfigManager>,
    /// Event publisher for application-wide events
    event_publisher: EventPublisher,
    /// IoC container integration
    container: Arc<ComponentContainer>,
    /// Application properties cache
    properties_cache: DashMap<String, ConfigValue>,
}

impl ApplicationContext {
    /// Creates a new application context
    ///
    /// # Examples
    ///
    /// ```rust
    /// use verdure_context::ApplicationContext;
    ///
    /// let context = ApplicationContext::new();
    /// ```
    pub fn new() -> Self {
        Self {
            config_manager: Arc::new(ConfigManager::new()),
            event_publisher: EventPublisher::new(),
            container: Arc::new(ComponentContainer::new()),
            properties_cache: DashMap::new(),
        }
    }

    /// Creates a builder for constructing an application context
    ///
    /// # Examples
    ///
    /// ```rust
    /// use verdure_context::ApplicationContext;
    ///
    /// let context = ApplicationContext::builder()
    ///     .with_property("app.version", "1.0.0")
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn builder() -> ApplicationContextBuilder {
        ApplicationContextBuilder::new()
    }
    fn initialize_early(&self) -> ContextResult<()> {
        self.container.register_component(self.config_manager.clone());

        for factory in inventory::iter::<ConfigFactory> {
            let config_component = (factory.create_fn)(self.config_manager.clone())?;
            self.container.register_component(config_component);
        }
        Ok(())
    }
    /// Initializes the application context
    ///
    /// This method initializes the IoC container and performs any other
    /// necessary initialization steps.
    ///
    /// # Returns
    ///
    /// `Ok(())` if initialization succeeds, an error otherwise
    ///
    /// # Examples
    ///
    /// ```rust
    /// use verdure_context::ApplicationContext;
    ///
    /// let context = ApplicationContext::new();
    /// context.initialize().unwrap();
    /// ```
    pub fn initialize(&self) -> ContextResult<()> {
        self.initialize_early()?;
        // Publish context-initializing event at the start
        let initializing_event = ContextInitializingEvent {
            config_sources_count: self.config_manager.sources_count(),
            active_profiles_count: self
                .config_manager
                .profile_manager()
                .active_profiles()
                .len(),
            timestamp: std::time::SystemTime::now(),
        };
        self.event_publisher
            .publish_with_context(&initializing_event, self);

        // Initialize the IoC container
        self.container.initialize().map_err(|e| {
            ContextError::initialization_failed(format!(
                "IoC container initialization failed: {}",
                e
            ))
        })?;

        // Publish context initialized event at the end
        let initialized_event = ContextInitializedEvent {
            config_sources_count: self.config_manager.sources_count(),
            active_profiles_count: self
                .config_manager
                .profile_manager()
                .active_profiles()
                .len(),
            timestamp: std::time::SystemTime::now(),
        };
        self.event_publisher
            .publish_with_context(&initialized_event, self);

        Ok(())
    }

    /// Gets a configuration value by key
    ///
    /// # Arguments
    ///
    /// * `key` - The configuration key
    ///
    /// # Returns
    ///
    /// The configuration value as a string, or an empty string if not found
    ///
    /// # Examples
    ///
    /// ```rust
    /// use verdure_context::{ApplicationContext, ConfigSource};
    /// use std::collections::HashMap;
    ///
    /// let mut context = ApplicationContext::new();
    ///
    /// let mut props = HashMap::new();
    /// props.insert("app.name".to_string(), "MyApp".to_string());
    /// context.add_config_source(ConfigSource::Properties(props)).unwrap();
    ///
    /// assert_eq!(context.get_config("app.name"), "MyApp");
    /// ```
    pub fn get_config(&self, key: &str) -> String {
        self.config_manager.get_string_or_default(key, "")
    }

    /// Gets a configuration value as a specific type
    ///
    /// # Arguments
    ///
    /// * `key` - The configuration key
    ///
    /// # Returns
    ///
    /// The configuration value parsed as the requested type
    ///
    /// # Errors
    ///
    /// Returns an error if the key is not found or cannot be parsed as the requested type
    ///
    /// # Examples
    ///
    /// ```rust
    /// use verdure_context::{ApplicationContext, ConfigSource};
    /// use std::collections::HashMap;
    ///
    /// let mut context = ApplicationContext::new();
    ///
    /// let mut props = HashMap::new();
    /// props.insert("app.port".to_string(), "8080".to_string());
    /// context.add_config_source(ConfigSource::Properties(props)).unwrap();
    ///
    /// let port: i64 = context.get_config_as("app.port").unwrap();
    /// assert_eq!(port, 8080);
    /// ```
    pub fn get_config_as<T>(&self, key: &str) -> ContextResult<T>
    where
        T: std::str::FromStr,
        T::Err: std::fmt::Display,
    {
        let value = self.config_manager.get_string(key)?;
        value
            .parse::<T>()
            .map_err(|e| ContextError::invalid_configuration(key, e.to_string()))
    }

    /// Gets a configuration value with a default
    ///
    /// # Arguments
    ///
    /// * `key` - The configuration key
    /// * `default` - The default value to return if key is not found
    ///
    /// # Examples
    ///
    /// ```rust
    /// use verdure_context::ApplicationContext;
    ///
    /// let context = ApplicationContext::new();
    /// let port = context.get_config_or_default("app.port", "8080");
    /// assert_eq!(port, "8080");
    /// ```
    pub fn get_config_or_default(&self, key: &str, default: &str) -> String {
        self.config_manager.get_string_or_default(key, default)
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
    /// use verdure_context::ApplicationContext;
    ///
    /// let mut context = ApplicationContext::new();
    /// context.set_config("runtime.property", "runtime.value");
    ///
    /// assert_eq!(context.get_config("runtime.property"), "runtime.value");
    /// ```
    /// Sets a configuration property
    pub fn set_config(&self, key: &str, value: &str) {
        let old_value = self.get_config(key);
        let old_value_opt = if old_value.is_empty() {
            None
        } else {
            Some(old_value)
        };

        self.config_manager
            .set(key, ConfigValue::String(value.to_string()));

        let event = ConfigurationChangedEvent {
            key: key.to_string(),
            old_value: old_value_opt,
            new_value: value.to_string(),
            timestamp: std::time::SystemTime::now(),
        };
        self.event_publisher.publish(&event);
    }

    /// Adds a configuration source
    pub fn add_config_source(&self, source: ConfigSource) -> ContextResult<()> {
        self.config_manager.add_source(source)
    }

    /// Gets the IoC container
    ///
    /// # Returns
    ///
    /// A reference to the IoC container
    ///
    /// # Examples
    ///
    /// ```rust
    /// use verdure_context::ApplicationContext;
    ///
    /// let context = ApplicationContext::new();
    /// let container = context.container();
    /// ```
    pub fn container(&self) -> Arc<ComponentContainer> {
        self.container.clone()
    }
    
    /// Gets a shared reference to the ConfigManager for IoC registration
    pub fn config_manager(&self) -> Arc<ConfigManager> {
        self.config_manager.clone()
    }

    /// Gets a component from the IoC container
    ///
    /// # Returns
    ///
    /// The requested component if found
    ///
    /// # Examples
    ///
    /// ```rust
    /// use verdure_context::ApplicationContext;
    /// use std::sync::Arc;
    ///
    /// #[derive(Debug)]
    /// struct MyService {
    ///     name: String,
    /// }
    ///
    /// let context = ApplicationContext::new();
    ///
    /// // First register the component
    /// let service = Arc::new(MyService {
    ///     name: "TestService".to_string(),
    /// });
    /// context.container().register_component(service);
    ///
    /// // Then retrieve it
    /// let retrieved: Option<Arc<MyService>> = context.get_component();
    /// assert!(retrieved.is_some());
    /// ```
    pub fn get_component<T: 'static + Send + Sync>(&self) -> Option<Arc<T>> {
        self.container.get_component()
    }

    /// Publishes an event
    ///
    /// # Arguments
    ///
    /// * `event` - The event to publish
    ///
    /// # Examples
    ///
    /// ```rust
    /// use verdure_context::{ApplicationContext, Event};
    /// use std::any::Any;
    ///
    /// #[derive(Debug, Clone)]
    /// struct MyEvent {
    ///     message: String,
    /// }
    ///
    /// impl Event for MyEvent {
    ///     fn name(&self) -> &'static str { "MyEvent" }
    ///     fn as_any(&self) -> &dyn Any { self }
    ///     fn into_any(self: Box<Self>) -> Box<dyn Any> { self }
    /// }
    ///
    /// let context = ApplicationContext::new();
    /// let event = MyEvent {
    ///     message: "Hello, World!".to_string(),
    /// };
    ///
    /// context.publish_event(&event);
    /// ```
    pub fn publish_event<T: Event + 'static>(&self, event: &T) {
        self.event_publisher.publish(event);
    }

    /// Subscribes to events with context access
    ///
    /// Context-aware listeners receive both the event and a reference to the ApplicationContext,
    /// allowing them to interact with the context during event handling.
    ///
    /// # Arguments
    ///
    /// * `listener` - The context-aware event listener to register
    ///
    /// # Examples
    ///
    /// ```rust
    /// use verdure_context::{ApplicationContext, ContextInitializedEvent, ContextAwareEventListener};
    ///
    /// struct StartupListener;
    ///
    /// impl ContextAwareEventListener<ContextInitializedEvent> for StartupListener {
    ///     fn on_context_event(&self, event: &ContextInitializedEvent, context: &ApplicationContext) {
    ///         println!("Context initialized! App name: {}", context.get_config("app.name"));
    ///         println!("Active profiles: {:?}", context.active_profiles());
    ///     }
    /// }
    ///
    /// let mut context = ApplicationContext::new();
    /// context.subscribe_to_context_events(StartupListener);
    /// ```
    /// Subscribes to events with context access
    pub fn subscribe_to_context_events<
        T: Event + 'static,
        L: ContextAwareEventListener<T> + 'static,
    >(
        &self,
        listener: L,
    ) {
        self.event_publisher.subscribe_context_aware(listener);
    }
    ///
    /// # Arguments
    ///
    /// * `listener` - The event listener to register
    ///
    /// # Examples
    ///
    /// ```rust
    /// use verdure_context::{ApplicationContext, Event, EventListener};
    /// use std::any::Any;
    ///
    /// #[derive(Debug, Clone)]
    /// struct TestEvent;
    ///
    /// impl Event for TestEvent {
    ///     fn name(&self) -> &'static str { "TestEvent" }
    ///     fn as_any(&self) -> &dyn Any { self }
    ///     fn into_any(self: Box<Self>) -> Box<dyn Any> { self }
    /// }
    ///
    /// struct TestListener;
    ///
    /// impl EventListener<TestEvent> for TestListener {
    ///     fn on_event(&self, _event: &TestEvent) {
    ///         println!("Event received!");
    ///     }
    /// }
    ///
    /// let mut context = ApplicationContext::new();
    /// context.subscribe_to_events(TestListener);
    /// ```
    /// Subscribes to events
    pub fn subscribe_to_events<T: Event + 'static, L: EventListener<T> + 'static>(
        &self,
        listener: L,
    ) {
        self.event_publisher.subscribe(listener);
    }

    /// Gets the active profiles
    pub fn active_profiles(&self) -> Vec<String> {
        self.config_manager.profile_manager().active_profiles().to_vec()
    }

    /// Checks if a specific profile is active
    ///
    /// # Arguments
    ///
    /// * `profile_name` - The profile name to check
    ///
    /// # Returns
    ///
    /// `true` if the profile is active, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```rust
    /// use verdure_context::{ApplicationContext, Profile};
    /// use std::collections::HashMap;
    ///
    /// // Create the profiles first
    /// let prod_profile = Profile::new("production", HashMap::new());
    /// let dev_profile = Profile::new("development", HashMap::new());
    ///
    /// let context = ApplicationContext::builder()
    ///     .with_profile(prod_profile)
    ///     .with_profile(dev_profile)
    ///     .with_active_profile("production")
    ///     .build()
    ///     .unwrap();
    ///
    /// assert!(context.is_profile_active("production"));
    /// assert!(!context.is_profile_active("development"));
    /// ```
    pub fn is_profile_active(&self, profile_name: &str) -> bool {
        self.active_profiles().contains(&profile_name.to_string())
    }

    /// Gets environment information
    ///
    /// # Returns
    ///
    /// A string representing the current environment (based on active profiles)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use verdure_context::{ApplicationContext, Profile};
    /// use std::collections::HashMap;
    ///
    /// // Create the profile first
    /// let profile = Profile::new("staging", HashMap::new());
    ///
    /// let context = ApplicationContext::builder()
    ///     .with_profile(profile)
    ///     .with_active_profile("staging")
    ///     .build()
    ///     .unwrap();
    ///
    /// println!("Environment: {}", context.environment());
    /// ```
    pub fn environment(&self) -> String {
        let profiles = self.active_profiles();
        if profiles.is_empty() {
            "default".to_string()
        } else {
            profiles.join(",")
        }
    }
}

impl Default for ApplicationContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_application_context_creation() {
        let context = ApplicationContext::new();
        assert_eq!(context.active_profiles().len(), 0);
    }

    #[test]
    fn test_application_context_builder() {
        let mut props = std::collections::HashMap::new();
        props.insert("test_key".to_string(), "test_value".to_string());
        let profile = Profile::new("test", props);

        let context = ApplicationContext::builder()
            .with_profile(profile)
            .with_active_profile("test")
            .with_property("app.name", "TestApp")
            .build()
            .unwrap();

        assert_eq!(context.active_profiles(), vec!["test"]);
        assert_eq!(context.get_config("app.name"), "TestApp");
    }

    #[test]
    fn test_configuration_management() {
        let context = ApplicationContext::new();

        let mut props = HashMap::new();
        props.insert(
            "database.url".to_string(),
            "postgres://localhost/test".to_string(),
        );
        props.insert("server.port".to_string(), "3000".to_string());
        props.insert("debug.enabled".to_string(), "true".to_string());

        context
            .add_config_source(ConfigSource::Properties(props))
            .unwrap();

        assert_eq!(
            context.get_config("database.url"),
            "postgres://localhost/test"
        );

        let port: i64 = context.get_config_as("server.port").unwrap();
        assert_eq!(port, 3000);

        let debug: bool = context.get_config_as("debug.enabled").unwrap();
        assert_eq!(debug, true);
    }

    #[test]
    fn test_configuration_with_defaults() {
        let context = ApplicationContext::new();

        assert_eq!(context.get_config("missing.key"), "");
        assert_eq!(
            context.get_config_or_default("missing.key", "default"),
            "default"
        );
    }

    #[test]
    fn test_runtime_configuration() {
        let context = ApplicationContext::new();

        context.set_config("runtime.property", "runtime.value");
        assert_eq!(context.get_config("runtime.property"), "runtime.value");
    }

    #[test]
    fn test_profile_management() {
        let mut dev_props = HashMap::new();
        dev_props.insert("env".to_string(), "dev".to_string());
        let dev_profile = Profile::new("development", dev_props);

        let mut local_props = HashMap::new();
        local_props.insert("location".to_string(), "local".to_string());
        let local_profile = Profile::new("local", local_props);

        let context = ApplicationContext::builder()
            .with_profile(dev_profile)
            .with_profile(local_profile)
            .with_active_profile("development")
            .with_active_profile("local")
            .build()
            .unwrap();

        assert_eq!(context.active_profiles(), vec!["development", "local"]);
        assert!(context.is_profile_active("development"));
        assert!(context.is_profile_active("local"));
        assert!(!context.is_profile_active("production"));

        assert_eq!(context.environment(), "development,local");
    }

    #[test]
    fn test_environment_default() {
        let context = ApplicationContext::new();
        assert_eq!(context.environment(), "default");
    }

    #[test]
    fn test_container_integration() {
        let context = ApplicationContext::new();
        let container = context.container();

        // Test that we can access the container
        assert!(container.get_component::<String>().is_none());
    }

    // Test event system integration
    use std::any::Any;

    #[derive(Debug, Clone)]
    struct TestContextEvent {
        message: String,
    }

    impl Event for TestContextEvent {
        fn name(&self) -> &'static str {
            "TestContextEvent"
        }

        fn as_any(&self) -> &dyn Any {
            self
        }

        fn into_any(self: Box<Self>) -> Box<dyn Any> {
            self
        }
    }

    struct TestContextListener {
        received: Arc<std::sync::Mutex<Vec<String>>>,
    }

    impl EventListener<TestContextEvent> for TestContextListener {
        fn on_event(&self, event: &TestContextEvent) {
            let mut received = self.received.lock().unwrap();
            received.push(event.message.clone());
        }
    }

    #[test]
    fn test_event_system_integration() {
        let received = Arc::new(std::sync::Mutex::new(Vec::new()));
        let listener = TestContextListener {
            received: received.clone(),
        };

        let context = ApplicationContext::new();
        context.subscribe_to_events(listener);

        let event = TestContextEvent {
            message: "test message".to_string(),
        };

        context.publish_event(&event);

        let received_messages = received.lock().unwrap();
        assert_eq!(received_messages.len(), 1);
        assert_eq!(received_messages[0], "test message");
    }

    #[test]
    fn test_built_in_context_events() {
        use crate::event::{
            ConfigurationChangedEvent, ContextInitializedEvent, ContextInitializingEvent,
            ProfileActivatedEvent,
        };
        use std::sync::{Arc, Mutex};

        // Event collectors
        let initializing_events = Arc::new(Mutex::new(Vec::new()));
        let initialized_events = Arc::new(Mutex::new(Vec::new()));
        let profile_events = Arc::new(Mutex::new(Vec::new()));
        let config_events = Arc::new(Mutex::new(Vec::new()));

        // Event listeners
        struct InitializingListener(Arc<Mutex<Vec<ContextInitializingEvent>>>);
        impl EventListener<ContextInitializingEvent> for InitializingListener {
            fn on_event(&self, event: &ContextInitializingEvent) {
                let mut events = self.0.lock().unwrap();
                events.push(event.clone());
            }
        }

        struct InitializedListener(Arc<Mutex<Vec<ContextInitializedEvent>>>);
        impl EventListener<ContextInitializedEvent> for InitializedListener {
            fn on_event(&self, event: &ContextInitializedEvent) {
                let mut events = self.0.lock().unwrap();
                events.push(event.clone());
            }
        }

        struct ProfileListener(Arc<Mutex<Vec<ProfileActivatedEvent>>>);
        impl EventListener<ProfileActivatedEvent> for ProfileListener {
            fn on_event(&self, event: &ProfileActivatedEvent) {
                let mut events = self.0.lock().unwrap();
                events.push(event.clone());
            }
        }

        struct ConfigListener(Arc<Mutex<Vec<ConfigurationChangedEvent>>>);
        impl EventListener<ConfigurationChangedEvent> for ConfigListener {
            fn on_event(&self, event: &ConfigurationChangedEvent) {
                let mut events = self.0.lock().unwrap();
                events.push(event.clone());
            }
        }

        // Create context with profile
        let mut dev_props = HashMap::new();
        dev_props.insert("env".to_string(), "development".to_string());
        dev_props.insert("debug".to_string(), "true".to_string());
        let dev_profile = Profile::new("development", dev_props);

        let mut context = ApplicationContext::builder()
            .with_profile(dev_profile)
            .with_active_profile("development")
            .with_property("initial.key", "initial.value")
            .build()
            .unwrap();

        // Subscribe to events BEFORE initialization
        context.subscribe_to_events(InitializingListener(initializing_events.clone()));
        context.subscribe_to_events(InitializedListener(initialized_events.clone()));
        context.subscribe_to_events(ProfileListener(profile_events.clone()));
        context.subscribe_to_events(ConfigListener(config_events.clone()));

        // Initialize context (should fire both ContextInitializingEvent and ContextInitializedEvent)
        context.initialize().unwrap();

        // Change configuration (should fire ConfigurationChangedEvent)
        context.set_config("runtime.key", "runtime.value");
        context.set_config("initial.key", "updated.value");

        // Verify initializing events were fired
        let initializing_events = initializing_events.lock().unwrap();
        assert_eq!(initializing_events.len(), 1);
        assert!(initializing_events[0].config_sources_count > 0);
        assert_eq!(initializing_events[0].active_profiles_count, 1);

        // Verify initialized events were fired
        let initialized_events = initialized_events.lock().unwrap();
        assert_eq!(initialized_events.len(), 1);
        assert!(initialized_events[0].config_sources_count > 0);
        assert_eq!(initialized_events[0].active_profiles_count, 1);

        // Verify the initializing event was fired before the initialized event
        assert!(initializing_events[0].timestamp <= initialized_events[0].timestamp);

        let profile_events = profile_events.lock().unwrap();
        assert_eq!(profile_events.len(), 0); // Profile event fired before subscription

        let config_events = config_events.lock().unwrap();
        assert_eq!(config_events.len(), 2);

        // First config change (new key)
        assert_eq!(config_events[0].key, "runtime.key");
        assert_eq!(config_events[0].old_value, None);
        assert_eq!(config_events[0].new_value, "runtime.value");

        // Second config change (update existing key)
        assert_eq!(config_events[1].key, "initial.key");
        assert_eq!(
            config_events[1].old_value,
            Some("initial.value".to_string())
        );
        assert_eq!(config_events[1].new_value, "updated.value");
    }

    #[test]
    fn test_context_aware_event_listeners() {
        use crate::event::{ContextAwareEventListener, ContextInitializedEvent};
        use std::sync::{Arc, Mutex};

        // Track events and context access
        let events_received = Arc::new(Mutex::new(Vec::new()));
        let context_data_accessed = Arc::new(Mutex::new(Vec::new()));

        // Context-aware event listener
        struct ContextAwareListener {
            events: Arc<Mutex<Vec<String>>>,
            context_data: Arc<Mutex<Vec<String>>>,
        }

        impl ContextAwareEventListener<ContextInitializedEvent> for ContextAwareListener {
            fn on_context_event(
                &self,
                event: &ContextInitializedEvent,
                context: &crate::context::ApplicationContext,
            ) {
                // Record the event
                let mut events = self.events.lock().unwrap();
                events.push(format!(
                    "Initialized with {} sources",
                    event.config_sources_count
                ));

                // Access context data
                let mut context_data = self.context_data.lock().unwrap();
                context_data.push(context.get_config("test.key"));
                context_data.push(context.environment());
                context_data.push(format!("profiles: {:?}", context.active_profiles()));
            }
        }

        // Create context with some configuration
        let mut context = ApplicationContext::builder()
            .with_property("test.key", "test.value")
            .with_property("app.name", "TestApp")
            .build()
            .unwrap();

        // Register context-aware listener
        let listener = ContextAwareListener {
            events: events_received.clone(),
            context_data: context_data_accessed.clone(),
        };
        context.subscribe_to_context_events(listener);

        // Initialize context - this should trigger the context-aware event
        context.initialize().unwrap();

        // Verify the context-aware listener received the event and could access context
        let events = events_received.lock().unwrap();
        assert_eq!(events.len(), 1);
        assert!(events[0].contains("Initialized with"));

        let context_data = context_data_accessed.lock().unwrap();
        assert_eq!(context_data.len(), 3);
        assert_eq!(context_data[0], "test.value"); // Successfully accessed config
        assert_eq!(context_data[1], "default"); // Successfully accessed environment
        assert_eq!(context_data[2], "profiles: []"); // Successfully accessed profiles
    }
}
