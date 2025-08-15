//! # Verdure - An Ecosystem Framework for Rust
//!
//! Verdure is a comprehensive **ecosystem framework** for Rust, Verdure aims to be the foundation
//! for building robust, scalable, and maintainable Rust applications across various domains.
//!
//! True to its name, Verdure represents a vibrant and thriving ecosystem framework, dedicated to
//! facilitating convenient and efficient Rust development through a cohesive set of tools and patterns.
//!
//! ## Framework Philosophy
//!
//! Verdure follows the **"Convention over Configuration"** and **"Batteries Included"** philosophies:
//!
//! - **Opinionated yet Flexible**: Provides sensible defaults while allowing customization
//! - **Ecosystem Integration**: Seamless integration between different framework modules  
//! - **Developer Experience**: Focus on developer productivity and code maintainability
//! - **Production Ready**: Built for real-world applications with performance and reliability in mind
//!
//! ## Ecosystem Modules
//!
//! Verdure is architected as a modular ecosystem:
//!
//! ### Core Foundation
//! - ✅ **verdure-core**: Foundation types, error handling, and common utilities
//! - ✅ **verdure-ioc**: Dependency injection container and component management
//! - ✅ **verdure-macros**: Compile-time code generation and annotation processing
//!
//! ### Application Framework
//! - ✅ **verdure-context**: Application context and configuration management
//!
//! ### Planned Modules
//!
//! ### Web & Network (Planned)
//! - TODO
//! ### Data & Persistence (Planned)
//! - TODO
//!
//! ### Security & Authentication (Planned)
//! - TODO
//!
//! ### Integration & Messaging (Planned)
//! - TODO
//!
//! ### Observability & Operations (Planned)
//! - TODO
//!
//! ### Testing & Development (Planned)
//! - TODO
//!
//! ## Current Features (v0.0.5)
//!
//! The current release provides a comprehensive foundation with application context support:
//!
//! - ✅ **Dependency Injection**: Comprehensive IoC container with automatic resolution
//! - ✅ **Component Lifecycle**: Singleton and prototype scopes with lifecycle events  
//! - ✅ **Annotation-Driven**: `#[derive(Component)]` and `#[autowired]` for declarative configuration
//! - ✅ **Event System**: Container and component lifecycle event handling
//! - ✅ **Circular Dependency Detection**: Prevents infinite dependency loops
//! - ✅ **Thread Safety**: Full concurrent access support for multi-threaded applications
//! - ✅ **Application Context**: Comprehensive application context management and event system
//! - ✅ **Auto-Configuration**: Automatic configuration file reading and component assembly
//! - ✅ **Multi-Format Configuration**: YAML, TOML, and Properties file format support
//! - ✅ **Default Value Support**: `#[config_default]` and `#[config_default_t]` attributes
//!
//! ## Quick Start
//!
//! Add Verdure to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! verdure = "0.0.5"
//! inventory = "0.3"  # Required for component discovery
//! ```
//!
//! ### Building Your First Application
//!
//! #### ApplicationContext Approach (Recommended)
//!
//! Create a configuration file `application.yml`:
//!
//! ```yaml
//! server:
//!   name: MyApp
//!   port: 8080
//! database:
//!   host: localhost
//!   username: app_user
//!   password: secret123
//! ```
//!
//! ```rust,ignore
//! use verdure::{ApplicationContext, Configuration, Component};
//! use verdure::event::{ContextAwareEventListener, ContextInitializingEvent};
//! use std::sync::Arc;
//!
//! // Auto-loaded configuration
//! #[derive(Debug, Configuration)]
//! #[configuration("server")]
//! struct ServerConfig {
//!     name: Option<String>,
//!     #[config_default(8080)]
//!     port: Option<u32>,
//! }
//!
//! #[derive(Debug, Configuration)]
//! #[configuration("database")]
//! struct DatabaseConfig {
//!     #[config_default("localhost")]
//!     host: Option<String>,
//!     username: Option<String>,
//!     password: Option<String>,
//! }
//!
//! // Business components
//! #[derive(Component)]
//! struct UserService {
//!     #[autowired]
//!     repository: Arc<UserRepository>,
//! }
//!
//! #[derive(Component)]
//! struct UserRepository;
//!
//! // Application startup listener
//! struct AppStartupListener;
//!
//! impl ContextAwareEventListener<ContextInitializingEvent> for AppStartupListener {
//!     fn on_context_event(
//!         &self,
//!         _event: &ContextInitializingEvent,
//!         context: &ApplicationContext
//!     ) {
//!         // Access configuration components
//!         let server_config = context.get_component::<ServerConfig>().expect("ServerConfig not found");
//!         let db_config = context.get_component::<DatabaseConfig>().expect("DatabaseConfig not found");
//!         
//!         println!("Starting {} on port {}", 
//!                  server_config.unwrap().name.unwrap_or_default(),
//!                  server_config.unwrap().port.unwrap_or(8080));
//!         // Register components with the context, for example:
//!         // context.register_component(Arc::new(DataSource::init(db_config.clone())));
//!     }
//! }
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create and initialize application context
//!     let context = ApplicationContext::builder()
//!         .with_config_file("application.yml")
//!         .build()?;
//!     
//!     // Subscribe to context events
//!     context.subscribe_to_context_events(AppStartupListener);
//!     
//!     // Initialize the context (auto-loads configs, wires dependencies)
//!     context.initialize()?;
//!     
//!     // Get your services with all dependencies and config injected
//!     let user_service: Arc<UserService> = context
//!         .get_component()
//!         .ok_or("UserService not found")?;
//!     
//!     // Your application is ready!
//!     Ok(())
//! }
//! ```
//!
//! #### Traditional IoC Container Approach
//!
//! ```rust,ignore
//! use verdure::{Component, ComponentContainer, ComponentFactory};
//! use std::sync::Arc;
//!
//! #[derive(Component)]
//! struct UserService {
//!     #[autowired]
//!     repository: Arc<UserRepository>,
//! }
//!
//! #[derive(Component)]
//! struct UserRepository;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let container = ComponentContainer::new();
//!     container.initialize()?;
//!     
//!     let user_service: Arc<UserService> = container
//!         .get_component()
//!         .ok_or("UserService not found")?;
//!     
//!     Ok(())
//! }
//! ```
//!
//! ## Roadmap & Vision
//!
//! Verdure aims to become the a comprehensive ecosystem
//! that covers all aspects of enterprise application development:
//!
//! ### Phase 1: Foundation (✅ Complete - v0.0.5)
//! - Core IoC container and dependency injection
//! - Component lifecycle and event system
//! - Application context management
//! - Auto-configuration and configuration management
//!
//! ### Phase 2: Enhanced Application Framework (v0.1.x)
//! - Advanced configuration profiles and environments
//! - Application bootstrapping enhancements
//! - Enhanced event system with more lifecycle events
//!
//! ### Phase 3: Web & Data (v0.2.x)  
//! - Full-featured web framework with MVC patterns
//! - Data access patterns and ORM integration
//! - Transaction management and caching
//!
//! ### Phase 4: Enterprise Features (v0.3.x+)
//! - Security and authentication framework
//! - Message-driven architecture and integration patterns
//! - Observability and production-ready tools
//!
//! ## Design Principles
//!
//! 1. **Type Safety**: Leverage Rust's type system for compile-time guarantees
//! 2. **Zero-Cost Abstractions**: Performance should not be sacrificed for convenience
//! 3. **Ecosystem Coherence**: All modules work together seamlessly
//! 4. **Convention over Configuration**: Sensible defaults with customization options
//! 5. **Developer Experience**: Focus on productivity and code maintainability
//! 6. **Production Ready**: Built for real-world, high-performance applications
//!
//! ## Community & Contribution
//!
//! Verdure is designed to be a community-driven ecosystem framework. We welcome contributions
//! across all modules and encourage the development of third-party extensions that integrate
//! with the Verdure ecosystem.
//!
//! Join us in building the future of Rust application development!

// Re-export the Component derive macro
pub use verdure_macros::Component;
pub use verdure_macros::Configuration;

// Re-export error handling types
pub use verdure_core::error;

// Re-export the lifecycle_listener macro
pub use verdure_ioc::lifecycle_listener;

// Re-export all IoC container types and traits
pub use verdure_ioc::{
    ComponentContainer, ComponentDefinition, ComponentFactory, ComponentInitializer,
    ComponentInstance, ComponentScope, ContainerLifecycleEvent, LifecycleEventPublisher,
    LifecycleListener, LifecycleListenerDefinition,
};

// Re-export context module types and traits
pub use verdure_context::{ApplicationContext, ContextResult, event, config};
