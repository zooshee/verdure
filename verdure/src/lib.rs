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
//! ### Application Framework (Planned)
//! - 🚧 **verdure-context**: Application context and environment management
//！- TODO
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
//! ## Current Features (v0.0.1)
//!
//! The initial release focuses on the foundational IoC container and component system:
//!
//! - ✅ **Dependency Injection**: Comprehensive IoC container with automatic resolution
//! - ✅ **Component Lifecycle**: Singleton and prototype scopes with lifecycle events  
//! - ✅ **Annotation-Driven**: `#[derive(Component)]` and `#[autowired]` for declarative configuration
//! - ✅ **Event System**: Container and component lifecycle event handling
//! - ✅ **Circular Dependency Detection**: Prevents infinite dependency loops
//! - ✅ **Thread Safety**: Full concurrent access support for multi-threaded applications
//! - ✅ **Zero-Cost Abstractions**: Compile-time dependency resolution where possible
//!
//! ## Quick Start
//!
//! Add Verdure to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! verdure = "0.0.1"
//! inventory = "0.3"  # Required for component discovery
//! ```
//!
//! ### Building Your First Application
//!
//! ```rust,ignore
//! use verdure::{Component, ComponentContainer, ComponentFactory};
//! use std::sync::Arc;
//!
//! // Domain layer - business logic
//! #[derive(Component)]
//! struct UserService {
//!     #[autowired]
//!     repository: Arc<UserRepository>,
//!     #[autowired]
//!     email_service: Arc<EmailService>,
//! }
//!
//! impl UserService {
//!     pub fn create_user(&self, email: &str) -> Result<User, UserError> {
//!         let user = self.repository.save(User::new(email))?;
//!         self.email_service.send_welcome_email(&user)?;
//!         Ok(user)
//!     }
//! }
//!
//! // Infrastructure layer - data access
//! #[derive(Component)]
//! struct UserRepository {
//!     connection_pool: DatabasePool,
//! }
//!
//! // Service layer - external integrations
//! #[derive(Component)]
//! struct EmailService {
//!     smtp_client: SmtpClient,
//! }
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Bootstrap the application
//!     let container = ComponentContainer::new();
//!     container.initialize()?;
//!     
//!     // The container automatically wires all dependencies
//!     let user_service: Arc<UserService> = container
//!         .get_component()
//!         .ok_or("UserService not found")?;
//!     
//!     // Business logic with all dependencies injected
//!     let user = user_service.create_user("user@example.com")?;
//!     println!("Created user: {}", user.email);
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
//! ### Phase 1: Foundation (Current - v0.0.x)
//! - Core IoC container and dependency injection
//! - Component lifecycle and event system
//!
//! ### Phase 2: Application Framework (v0.1.x)
//! - Basic application context management
//! - Auto-configuration and application bootstrapping
//! - Configuration management and profiles
//! - Enhanced application context with environment support
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
