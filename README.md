# Verdure

[![Crates.io version](https://img.shields.io/crates/v/verdure.svg?style=flat-square)](https://crates.io/crates/verdure)
[![docs.rs docs](https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square)](https://docs.rs/verdure)

English | [简体中文](./README-zh_cn.md)

Verdure - An ecosystem framework for Rust

True to its name, Verdure represents a vibrant and thriving ecosystem framework, dedicated to facilitating convenient and efficient Rust development through a comprehensive, integrated suite of tools and patterns.

The project is currently in the foundational development phase. We are looking for enthusiastic contributors to join us in building it.

## Ecosystem Modules

### ✅ Current Release (v0.0.1) - Foundation

- **verdure-core**: Foundation types, error handling, and common utilities
- **verdure-ioc**: Dependency injection container and component management  
- **verdure-macros**: Compile-time code generation and annotation processing
- **verdure-context**: Application context and environment management (🚧)

### 🚧 Upcoming Releases - Complete Ecosystem

**Application Framework**:
- verdure-boot: Auto-configuration and application bootstrapping
- verdure-config: Configuration management and property binding
- verdure-profiles: Environment-specific configuration profiles

**Web & Network**:
- verdure-web: Web framework with MVC patterns
- verdure-http: HTTP client and server abstractions
- verdure-websocket: WebSocket support and real-time communication

**Data & Persistence**:
- verdure-data: Data access patterns and repository abstractions
- verdure-orm: Object-relational mapping with active record patterns
- verdure-transaction: Transaction management and ACID support

**Security & Authentication**:
- verdure-security: Authentication and authorization framework
- verdure-oauth: OAuth2 and OpenID Connect integration


## Current Features (v0.0.1)

- [x] **IoC Container**: Comprehensive dependency injection with automatic resolution
- [x] **Component Lifecycle**: Singleton and prototype scopes with lifecycle events
- [x] **Annotation-Driven Development**: `#[derive(Component)]` and `#[autowired]` for declarative configuration
- [x] **Event System**: Container and component lifecycle event handling
- [x] **Circular Dependency Detection**: Prevents infinite dependency loops
- [x] **Thread Safety**: Full concurrent access support for multi-threaded applications

### 📋 Roadmap - Building the Complete Ecosystem

- [ ] **Auto-Configuration**: Out-of-the-box application bootstrapping
- [ ] **Web Framework**: MVC patterns and REST API development
- [ ] **Data Access**: Repository patterns and ORM integration
- [ ] **Security Framework**: Authentication and authorization
- [ ] **AOP (Aspect-Oriented Programming)**: Cross-cutting concern support
- [ ] **Message-Driven Architecture**: Event-driven programming patterns
- [ ] **Observability**: Metrics, tracing, and health checks
- [ ] And much more...


## Add Dependency

```toml
verdure = "0.0.1"
inventory = "0.3"
```
The underlying implementation heavily relies on inventory. Our thanks go to the authors of this excellent repository.

## IoC / DI

### Initialize the Container
```rust
use std::sync::Arc;

fn init_container() {
    let container = ComponentContainer::new();
    match container.initialize() {
        Ok(_) => Arc::new(container),
        Err(e) => panic!("Failed to initialize container {}", e)
    }
}
```
### Register a Component
#### Automatic Registration and Injection (Derive)
Adding the `#[derive(Component)]` macro to a struct automatically registers it with the container as a singleton by default. For fields marked with the `#[autowired]` attribute, an instance will be automatically retrieved from the container and injected.
```rust
use verdure::Component;

#[derive(Component)]
struct TestA {
    #[autowired]
    test_b: Arc<TestB>,
    test_c: Option<TestC>,
    test_d: TestD
}

#[derive(Component)]
struct TestB {
    a: i32,
    b: i32,
}

struct TestC {
    a: i32
}

#[derive(Default)]
struct TestD {
    a: i32,
}
```
There are two important points to note:
* The field to be injected must be wrapped in an `Arc<T>`.
* Fields that do not require injection must either be of type `Option<T> ` or implement the `Default` trait.

#### Manual Registration and Component Retrieval
```rust
#[derive(Debug)]
struct Config {
    name: &'static str,
    port: u16
}

fn main() {
    let container = init_container();
    container.register_component(Arc::new(config));
    let config = container.get_component::<Config>().unwrap();
    println!("config: {:?}", config);
}
```

### Container Event Listening
#### Using the Macro
```rust
fn handle_container_lifecycle(event: &ContainerLifecycleEvent) {
    match event {
        ContainerLifecycleEvent::InitializationStarted {
            container,
            component_count,
        } => {
            // You can register necessary components during initialization here.
        }
        ContainerLifecycleEvent::InitializationCompleted {
            container: _,
            component_count,
            duration,
        } => {
            println!(
                "Container initialization completed\nComponent count: {}\nTime taken: {:?}",
                component_count, duration
            );
        }
        ContainerLifecycleEvent::ComponentCreated {
            container: _,
            component_name,
            component_type_id,
            creation_duration,
        } => {
            println!(
                "Component created\nName: {}\nType ID: {:?}\nCreation time: {:?}",
                component_name, component_type_id, creation_duration
            );
        }
    }
}
lifecycle_listener!("app_container_listener", handle_container_lifecycle);
```