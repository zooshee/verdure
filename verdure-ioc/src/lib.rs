//! Verdure IoC Container - Core IoC Module of the Verdure Ecosystem
//!
//! This crate provides the foundational Inversion of Control (IoC) container functionality
//! for the **Verdure ecosystem framework**. As the core dependency injection engine of Verdure,
//! it enables the declarative, annotation-driven development model that powers the entire
//! ecosystem.
//!
//! ## Ecosystem Integration
//!
//! As a core module of the Verdure ecosystem, this IoC container integrates seamlessly with:
//!
//! - **verdure-web**: Web framework components and request handling
//! - **verdure-data**: Repository and data access layer components  
//! - **verdure-config**: Configuration-driven component initialization
//! - **verdure-security**: Authentication and authorization components
//! - **verdure-boot**: Auto-configuration and component discovery
//!
//! ## Core Features
//!
//! * **Ecosystem Foundation**: Powers dependency injection across all Verdure modules
//! * **Annotation-Driven**: `#[derive(Component)]` and `#[autowired]` for declarative configuration
//! * **Component Lifecycle**: Comprehensive lifecycle management with singleton and prototype scopes
//! * **Event System**: Container lifecycle events for monitoring and debugging
//! * **Circular Dependency Detection**: Prevents infinite dependency loops
//! * **Thread Safety**: Full support for multi-threaded applications
//! * **Zero-Cost Abstractions**: Compile-time dependency resolution where possible
//!
//! # Quick Start
//!
//! ```rust
//! use verdure_ioc::{ComponentContainer, ComponentFactory};
//! use std::sync::Arc;
//!
//! // Create a container
//! let container = ComponentContainer::new();
//!
//! // Register a component manually
//! #[derive(Debug)]
//! struct DatabaseService {
//!     connection_string: String,
//! }
//!
//! let db_service = Arc::new(DatabaseService {
//!     connection_string: "postgres://localhost:5432/db".to_string(),
//! });
//!
//! container.register_component(db_service);
//!
//! // Retrieve the component
//! let retrieved_service: Option<Arc<DatabaseService>> = container.get_component();
//! assert!(retrieved_service.is_some());
//! ```

mod component;
mod container;
mod event;

pub use component::{
    ComponentDefinition, ComponentInitializer, ComponentInstance, ComponentScope,
    factory::ComponentFactory,
};

pub use container::ComponentContainer;

pub use event::{
    ContainerLifecycleEvent, LifecycleEventPublisher, LifecycleListener,
    LifecycleListenerDefinition,
};

/// Macro for registering lifecycle event listeners
///
/// This macro simplifies the registration of lifecycle event listeners with the container.
///
/// # Arguments
///
/// * `$name` - A string literal identifying the listener
/// * `$handler` - A function that handles lifecycle events
///
/// # Examples
///
/// ```rust
/// use verdure_ioc::{lifecycle_listener, ContainerLifecycleEvent};
///
/// fn my_event_handler(event: &ContainerLifecycleEvent) {
///     match event {
///         ContainerLifecycleEvent::InitializationStarted { .. } => {
///             println!("Container initialization started");
///         }
///         _ => {}
///     }
/// }
///
/// lifecycle_listener!("my_listener", my_event_handler);
/// ```
#[macro_export]
macro_rules! lifecycle_listener {
    ($name:expr, $handler:expr) => {
        inventory::submit! {
            $crate::LifecycleListenerDefinition {
                name: $name,
                handler: $handler,
            }
        }
    };
}
