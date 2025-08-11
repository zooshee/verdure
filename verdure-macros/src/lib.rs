//! Verdure Procedural Macros - Annotation Processing for the Verdure Ecosystem
//!
//! This crate provides the procedural macros that enable Verdure's declarative,
//! annotation-driven programming model. These macros power the entire Verdure ecosystem
//! by automatically generating the boilerplate code needed for dependency injection,
//! component registration, and framework integration.
//!
//! As part of the Verdure ecosystem, these macros work seamlessly across all framework
//! modules - from web controllers to data repositories to security components.
//!
//! # Macros
//!
//! * `#[derive(Component)]` - Automatically implements `ComponentInitializer` and registers the component
//!
//! # Attributes
//!
//! * `#[autowired]` - Marks a field for automatic dependency injection
//! * `#[component]` - Provides component configuration options
//!
//! # Examples
//!
//! ```rust
//! use verdure::Component;
//! use std::sync::Arc;
//!
//! #[derive(Component)]
//! struct DatabaseService {
//!     connection_string: String,
//! }
//!
//! #[derive(Component)]
//! struct UserService {
//!     #[autowired]
//!     db: Arc<DatabaseService>,
//!     cache_size: usize,
//! }
//! ```
//!
//! The `#[derive(Component)]` macro will:
//!
//! 1. Generate an implementation of `ComponentInitializer`
//! 2. Register the component with the global component registry
//! 3. Automatically handle dependency injection for `#[autowired]` fields
//! 4. Initialize non-autowired fields using `Default::default()` or `None` for `Option<T>`

mod component;

use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

/// Derive macro for automatic component registration and dependency injection
///
/// The `Component` derive macro automatically implements the `ComponentInitializer` trait
/// and registers the component with the IoC container. It handles dependency injection
/// for fields marked with `#[autowired]` and provides sensible defaults for other fields.
///
/// # Attributes
///
/// * `#[autowired]` - Marks a field for automatic dependency injection. The field must be of type `Arc<T>`
/// * `#[component(scope = "...")]` - Sets the component scope (defaults to `Singleton`)
///
/// # Field Initialization Rules
///
/// 1. **Autowired fields**: Automatically injected by the container
/// 2. **Option fields**: Initialized to `None`
/// 3. **Other fields**: Initialized using `Default::default()`
///
/// # Examples
///
/// ```rust
/// use verdure::Component;
/// use std::sync::Arc;
///
/// // Simple component with no dependencies
/// #[derive(Component)]
/// struct ConfigService {
///     config_path: String, // Will be initialized with Default::default() = ""
///     port: u16,           // Will be initialized with Default::default() = 0
/// }
///
/// // Component with dependencies
/// #[derive(Component)]
/// struct DatabaseService {
///     #[autowired]
///     config: Arc<ConfigService>, // Automatically injected
///     optional_cache: Option<String>, // Initialized to None
///     connection_pool_size: usize,    // Default::default() = 0
/// }
///
/// // Component with custom scope
/// #[derive(Component)]
/// #[component(scope = "Prototype")]
/// struct RequestHandler {
///     #[autowired]
///     db: Arc<DatabaseService>,
///     request_id: String,
/// }
/// ```
///
/// # Generated Code
///
/// The macro generates implementations of `ComponentInitializer` and registers the component
/// with the global registry using the `inventory` crate.
///
/// # Panics
///
/// The macro will produce compile-time errors in the following cases:
///
/// * Applying to enums or unions (only structs with named fields are supported)
/// * Using `#[autowired]` on fields that are not `Arc<T>`
/// * Invalid syntax in component attributes
#[proc_macro_derive(Component, attributes(component, autowired))]
pub fn component_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    component::impl_component_derive(&ast).into()
}
