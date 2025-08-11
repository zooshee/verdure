//! Verdure Application Context - Context Management for the Verdure Ecosystem
//!
//! This crate provides application context management as a core part of the Verdure ecosystem
//! framework. It serves as the central hub for application-wide state, configuration,
//! and environment management that integrates with all other Verdure modules.
//!
//! This module provides the foundation
//! for configuration-driven development and environment-aware component behavior
//! across the entire Verdure ecosystem.
//!
//! # Core Features
//!
//! * **Application Context**: Centralized application state management
//! * **Configuration Management**: Hierarchical configuration system with multiple sources
//! * **Environment Profiles**: Support for different deployment environments
//! * **Event Broadcasting**: Application-wide event system for decoupled communication
//! * **IoC Integration**: Seamless integration with the Verdure IoC container
//! * **Type-Safe Configuration**: Strongly-typed configuration value access
//!
//! # Quick Start
//!
//! ## Basic Usage
//!
//! ```rust
//! use verdure_context::{ApplicationContext, ConfigSource};
//! use std::collections::HashMap;
//!
//! // Create and configure application context
//! let context = ApplicationContext::builder()
//!     .with_property("app.name", "MyApp")
//!     .with_property("app.port", "8080")
//!     .build()
//!     .unwrap();
//!
//! // Initialize the context
//! context.initialize().unwrap();
//!
//! // Access configuration
//! let app_name = context.get_config("app.name");
//! let port: i64 = context.get_config_as("app.port").unwrap();
//!
//! println!("Starting {} on port {}", app_name, port);
//! ```
//!
//! ## Configuration from Files
//!
//! Verdure Context supports multiple configuration file formats:
//!
//! ### TOML Configuration
//!
//! ```rust,no_run
//! use verdure_context::ApplicationContext;
//!
//! let context = ApplicationContext::builder()
//!     .with_toml_config_file("config/app.toml")
//!     .build()
//!     .unwrap();
//! ```
//!
//! ### YAML Configuration
//!
//! ```rust,no_run
//! use verdure_context::ApplicationContext;
//!
//! let context = ApplicationContext::builder()
//!     .with_yaml_config_file("config/app.yaml")
//!     .build()
//!     .unwrap();
//! ```
//!
//! ### Properties Configuration
//!
//! ```rust,no_run
//! use verdure_context::ApplicationContext;
//!
//! let context = ApplicationContext::builder()
//!     .with_properties_config_file("config/app.properties")
//!     .build()
//!     .unwrap();
//! ```
//!
//! ### Auto-Detection
//!
//! ```rust,no_run
//! use verdure_context::ApplicationContext;
//!
//! // Format is auto-detected based on file extension
//! let context = ApplicationContext::builder()
//!     .with_config_file("config/app.yaml")      // YAML
//!     .with_config_file("config/db.properties") // Properties
//!     .with_config_file("config/server.toml")   // TOML
//!     .build()
//!     .unwrap();
//! ```
//!
//! ## Event System
//!
//! ```rust
//! use verdure_context::{ApplicationContext, Event, EventListener};
//! use std::any::Any;
//!
//! // Define an event
//! #[derive(Debug, Clone)]
//! struct UserRegisteredEvent {
//!     pub user_id: u64,
//!     pub email: String,
//! }
//!
//! impl Event for UserRegisteredEvent {
//!     fn name(&self) -> &'static str {
//!         "UserRegistered"
//!     }
//!     
//!     fn as_any(&self) -> &dyn Any {
//!         self
//!     }
//!     
//!     fn into_any(self: Box<Self>) -> Box<dyn Any> {
//!         self
//!     }
//! }
//!
//! // Create event listener
//! struct EmailNotificationListener;
//!
//! impl EventListener<UserRegisteredEvent> for EmailNotificationListener {
//!     fn on_event(&self, event: &UserRegisteredEvent) {
//!         println!("Sending welcome email to user {} ({})",
//!                  event.user_id, event.email);
//!     }
//! }
//!
//! // Set up context with event handling
//! let mut context = ApplicationContext::new();
//! context.subscribe_to_events(EmailNotificationListener);
//!
//! // Publish events
//! let event = UserRegisteredEvent {
//!     user_id: 123,
//!     email: "user@example.com".to_string(),
//! };
//! context.publish_event(&event);
//! ```
//!
//! ## IoC Container Integration
//!
//! ```rust
//! use verdure_context::ApplicationContext;
//! use std::sync::Arc;
//!
//! #[derive(Debug)]
//! struct DatabaseService {
//!     connection_url: String,
//! }
//!
//! let context = ApplicationContext::builder()
//!     .with_property("database.url", "postgres://localhost/myapp")
//!     .build()
//!     .unwrap();
//!
//! // Register components with the container
//! let db_service = Arc::new(DatabaseService {
//!     connection_url: context.get_config("database.url"),
//! });
//! context.container().register_component(db_service);
//!
//! // Retrieve components
//! let retrieved: Option<Arc<DatabaseService>> = context.get_component();
//! assert!(retrieved.is_some());
//! ```
//!
//! # Advanced Features
//!
//! ## Environment Profiles
//!
//! ```rust
//! use verdure_context::{ApplicationContext, Profile};
//! use std::collections::HashMap;
//!
//! // Create profile-specific configurations
//! let mut dev_props = HashMap::new();
//! dev_props.insert("database.url".to_string(), "postgres://localhost/dev".to_string());
//! dev_props.insert("logging.level".to_string(), "DEBUG".to_string());
//! let dev_profile = Profile::new("development", dev_props);
//!
//! let mut prod_props = HashMap::new();
//! prod_props.insert("database.url".to_string(), "postgres://prod-server/app".to_string());
//! prod_props.insert("logging.level".to_string(), "INFO".to_string());
//! let prod_profile = Profile::new("production", prod_props);
//!
//! let context = ApplicationContext::builder()
//!     .with_profile(dev_profile)
//!     .with_profile(prod_profile)
//!     .with_active_profile("development")
//!     .build()
//!     .unwrap();
//!
//! // Configuration values will be resolved from active profile
//! assert_eq!(context.get_config("logging.level"), "DEBUG");
//! ```
//!
//! ## Configuration Sources Priority
//!
//! Configuration sources are resolved in the following order (highest to lowest precedence):
//!
//! 1. **Runtime Properties**: Values set via `set_config()`
//! 2. **Active Profiles**: Profile-specific configuration properties
//! 3. **Configuration Sources**: Sources added via `add_config_source()` (last added wins)
//! 4. **Environment Variables**: System environment variables
//! 5. **Configuration Files**: Files loaded via various methods (last added wins)
//!    - TOML files (`.toml`)
//!    - YAML files (`.yaml`, `.yml`)
//!    - Properties files (`.properties`)
//!
//! ## Supported Configuration Formats
//!
//! ### TOML Format Example
//!
//! ```toml
//! # app.toml
//! [app]
//! name = "MyApplication"
//! port = 8080
//! debug = true
//!
//! [database]
//! host = "localhost"
//! port = 5432
//! name = "myapp"
//! ```
//!
//! ### YAML Format Example
//!
//! ```yaml
//! # app.yaml
//! app:
//!   name: MyApplication
//!   port: 8080
//!   debug: true
//!   features:
//!     - auth
//!     - logging
//!
//! database:
//!   host: localhost
//!   port: 5432
//!   name: myapp
//! ```
//!
//! ### Properties Format Example
//!
//! ```properties
//! # app.properties
//! app.name=MyApplication
//! app.port=8080
//! app.debug=true
//!
//! database.host=localhost
//! database.port=5432
//! database.name=myapp
//! ```
//!
//! All formats are converted to a flat key-value structure using dot notation
//! (e.g., `app.name`, `database.host`) for consistent access patterns.
//!
//! # Ecosystem Context Events
//!
//! The context system publishes several built-in events that applications can listen to.
//! There are two ways to listen to events:
//!
//! 1. **Regular Event Listeners**: Receive only the event data
//! 2. **Context-Aware Event Listeners**: Receive both the event and ApplicationContext reference
//!
//! ## Built-in Lifecycle Events
//!
//! ### ContextInitializingEvent
//!
//! **When**: Fired at the very beginning of context initialization, before any actual work begins.  
//! **Purpose**: Allows listeners to prepare for context startup or log initialization start.  
//! **Data**: Configuration sources count, active profiles count, and timestamp.
//!
//! ### ContextInitializedEvent  
//!
//! **When**: Fired after the context is fully initialized, including all configuration sources, profiles, and IoC container.  
//! **Purpose**: Ideal for application startup tasks that require a fully configured context.  
//! **Data**: Final configuration sources count, active profiles count, and timestamp.
//!
//! ### ProfileActivatedEvent
//!
//! **When**: Fired whenever a profile is activated during context building.  
//! **Purpose**: Allows listeners to react to environment changes or profile-specific setup.  
//! **Data**: Profile name, properties count in the profile, and timestamp.
//!
//! ### ConfigurationChangedEvent
//!
//! **When**: Fired when configuration values are updated at runtime after context initialization.  
//! **Purpose**: Enables reactive configuration updates and change tracking.  
//! **Data**: Configuration key, old value (if any), new value, and timestamp.
//!
//! ## Context-Aware Event Listeners (Recommended for Lifecycle Events)
//!
//! Context-aware listeners can access the ApplicationContext during event handling,
//! making them perfect for lifecycle events where you need to interact with the context:
//!
//! ```rust
//! use verdure_context::{ApplicationContext, ContextInitializedEvent, ContextAwareEventListener};
//!
//! struct StartupTasks;
//!
//! impl ContextAwareEventListener<ContextInitializedEvent> for StartupTasks {
//!     fn on_context_event(&self, event: &ContextInitializedEvent, context: &ApplicationContext) {
//!         println!("🚀 Context initialized with {} sources!", event.config_sources_count);
//!         
//!         // Access configuration
//!         let app_name = context.get_config("app.name");
//!         println!("📱 Starting application: {}", app_name);
//!         
//!         // Access IoC container for dependency injection setup
//!         let container = context.container();
//!         // Setup your components...
//!         
//!         // Check environment and profiles
//!         let env = context.environment();
//!         let profiles = context.active_profiles();
//!         println!("🌍 Running in {} environment with profiles: {:?}", env, profiles);
//!     }
//! }
//!
//! let mut context = ApplicationContext::builder()
//!     .with_property("app.name", "MyApp")
//!     .build()
//!     .unwrap();
//!
//! context.subscribe_to_context_events(StartupTasks);
//! context.initialize().unwrap(); // Triggers the context-aware listener
//! ```
//!
//! ## ContextInitializingEvent Usage
//!
//! Listen to preparation phases before initialization:
//!
//! ```rust
//! use verdure_context::{ApplicationContext, ContextInitializingEvent, ContextAwareEventListener};
//!
//! struct PreStartupListener;
//!
//! impl ContextAwareEventListener<ContextInitializingEvent> for PreStartupListener {
//!     fn on_context_event(&self, event: &ContextInitializingEvent, context: &ApplicationContext) {
//!         println!("🔧 Context initializing with {} sources and {} profiles...",
//!                  event.config_sources_count, event.active_profiles_count);
//!         
//!         // Pre-initialization tasks
//!         let startup_time = event.timestamp;
//!         println!("⏰ Startup began at: {:?}", startup_time);
//!     }
//! }
//! ```
//!
//! ## ProfileActivatedEvent Usage
//!
//! React to profile activation during context building:
//!
//! ```rust
//! use verdure_context::{ApplicationContext, ProfileActivatedEvent, EventListener, Profile};
//! use std::collections::HashMap;
//!
//! struct ProfileListener;
//!
//! impl EventListener<ProfileActivatedEvent> for ProfileListener {
//!     fn on_event(&self, event: &ProfileActivatedEvent) {
//!         println!("🔄 Profile '{}' activated with {} properties",
//!                  event.profile_name, event.properties_count);
//!     }
//! }
//!
//! let mut props = HashMap::new();
//! props.insert("env".to_string(), "production".to_string());
//! let profile = Profile::new("production", props);
//!
//! let context = ApplicationContext::builder()
//!     .with_profile(profile)
//!     .with_active_profile("production")
//!     .build()
//!     .unwrap();
//! // Profile activation event was published during build
//! ```
//!
//! ## ConfigurationChangedEvent Usage
//!
//! Track runtime configuration changes:
//!
//! ```rust
//! use verdure_context::{ApplicationContext, ConfigurationChangedEvent, EventListener};
//!
//! struct ConfigListener;
//!
//! impl EventListener<ConfigurationChangedEvent> for ConfigListener {
//!     fn on_event(&self, event: &ConfigurationChangedEvent) {
//!         match &event.old_value {
//!             Some(old) => println!("⚙️ Configuration '{}' changed from '{}' to '{}'",
//!                                   event.key, old, event.new_value),
//!             None => println!("➕ Configuration '{}' set to '{}'",
//!                              event.key, event.new_value),
//!         }
//!     }
//! }
//!
//! let mut context = ApplicationContext::new();
//! context.subscribe_to_events(ConfigListener);
//! context.set_config("app.mode", "production");
//! ```
//!
//! ## Event System Architecture
//!
//! The event system supports both regular and context-aware listeners simultaneously:
//!
//! - **Regular listeners** (`EventListener<T>`) receive only the event data
//! - **Context-aware listeners** (`ContextAwareEventListener<T>`) receive both event data and ApplicationContext reference
//! - Both types can be registered for the same event type
//! - Events are published to all registered listeners of the appropriate type
//! - Lifecycle events automatically provide context access for enhanced integration capabilities

pub mod config;
pub mod context;
pub mod error;
pub mod event;
pub mod profile;

// Re-export main types for convenience
pub use config::{ConfigManager, ConfigSource, ConfigValue};
pub use context::{ApplicationContext, ApplicationContextBuilder};
pub use error::{ContextError, ContextResult};
pub use event::{
    AnyContextAwareEventListener, AnyEventListener, ConfigurationChangedEvent,
    ContextAwareEventListener, ContextInitializedEvent, ContextInitializingEvent, Event,
    EventListener, EventPublisher, ProfileActivatedEvent,
};
pub use profile::{Profile, ProfileManager};
