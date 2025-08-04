# Verdure

[![Crates.io version](https://img.shields.io/crates/v/verdure.svg?style=flat-square)](https://crates.io/crates/verdure)
[![docs.rs docs](https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square)](https://docs.rs/verdure)

English | [简体中文](./README-zh_cn.md)

Verdure - An ecosystem framework for Rust.

True to its name, Verdure aims to be a vibrant and thriving ecosystem framework, dedicated to facilitating convenient and efficient Rust development.

The project is currently in the foundational development phase. We are looking for enthusiastic contributors to join us in building it.

## Features

- [x] IoC container and container event listening
- [x] Dependency Injection (DI)
- [ ] Automatic Configuration
- [ ] AOP (Aspect-Oriented Programming)
- [ ] Context
- [ ] And more...

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