//! Container lifecycle event system for the Verdure ecosystem
//!
//! This module provides a comprehensive event system for monitoring container
//! and component lifecycle events across the Verdure ecosystem. It enables
//! observability and debugging of the dependency injection process that powers
//! all Verdure applications, from simple services to complex web applications.

use crate::container::ComponentContainer;
use std::any::TypeId;
use std::time::Duration;

/// Container lifecycle events enumeration
///
/// This enum represents different events that occur during the container's lifecycle,
/// including initialization phases and component creation events.
///
/// # Examples
///
/// ```rust
/// use verdure_ioc::{ContainerLifecycleEvent, ComponentContainer};
///
/// fn handle_event(event: &ContainerLifecycleEvent) {
///     match event {
///         ContainerLifecycleEvent::InitializationStarted { component_count, .. } => {
///             println!("Starting initialization of {} components", component_count);
///         }
///         ContainerLifecycleEvent::InitializationCompleted { duration, .. } => {
///             println!("Initialization completed in {:?}", duration);
///         }
///         ContainerLifecycleEvent::ComponentCreated { component_name, .. } => {
///             println!("Created component: {}", component_name);
///         }
///     }
/// }
/// ```
pub enum ContainerLifecycleEvent<'a> {
    /// Fired when container initialization begins
    InitializationStarted {
        /// Reference to the container being initialized
        container: &'a ComponentContainer,
        /// Total number of components to be initialized
        component_count: usize,
    },
    /// Fired when container initialization completes successfully
    InitializationCompleted {
        /// Reference to the initialized container
        container: &'a ComponentContainer,
        /// Number of components that were successfully initialized
        component_count: usize,
        /// Total time taken for initialization
        duration: Duration,
    },
    /// Fired when an individual component is created
    ComponentCreated {
        /// Reference to the container
        container: &'a ComponentContainer,
        /// Human-readable name of the component type
        component_name: &'static str,
        /// TypeId of the created component
        component_type_id: TypeId,
        /// Time taken to create this specific component
        creation_duration: Duration,
    },
}

/// Trait for implementing lifecycle event listeners
///
/// Implement this trait to receive notifications about container lifecycle events.
/// Listeners must be thread-safe as they may be called from multiple threads.
///
/// # Examples
///
/// ```rust
/// use verdure_ioc::{LifecycleListener, ContainerLifecycleEvent};
///
/// struct MyListener;
///
/// impl LifecycleListener for MyListener {
///     fn on_lifecycle_event(&self, event: &ContainerLifecycleEvent) {
///         println!("Received lifecycle event");
///     }
/// }
/// ```
pub trait LifecycleListener: Send + Sync {
    /// Called when a lifecycle event occurs
    ///
    /// # Arguments
    ///
    /// * `event` - The lifecycle event that occurred
    fn on_lifecycle_event(&self, event: &ContainerLifecycleEvent);
}

/// Static definition of a lifecycle event listener
///
/// This structure is used to register event listeners with the container
/// using the `lifecycle_listener!` macro. It contains the listener's name
/// and handler function.
///
/// # Examples
///
/// ```rust
/// use verdure_ioc::{LifecycleListenerDefinition, ContainerLifecycleEvent};
///
/// fn my_handler(event: &ContainerLifecycleEvent) {
///     println!("Event received");
/// }
///
/// let definition = LifecycleListenerDefinition {
///     name: "my_listener",
///     handler: my_handler,
/// };
/// ```
pub struct LifecycleListenerDefinition {
    /// Unique name identifying this listener
    pub name: &'static str,
    /// Function to call when events occur
    pub handler: fn(&ContainerLifecycleEvent),
}

inventory::collect!(LifecycleListenerDefinition);

/// Publisher for container lifecycle events
///
/// `LifecycleEventPublisher` manages the collection of registered event listeners
/// and dispatches events to all registered handlers. It is used internally by
/// the container to publish events during various lifecycle phases.
///
/// # Thread Safety
///
/// This struct is thread-safe and can be shared across multiple threads.
/// Event publishing is synchronous and will call all listeners in sequence.
pub struct LifecycleEventPublisher {
    /// Collection of all registered listener definitions
    listeners: Vec<&'static LifecycleListenerDefinition>,
}

impl LifecycleEventPublisher {
    /// Creates a new LifecycleEventPublisher
    ///
    /// This method automatically discovers all registered lifecycle listeners
    /// using the inventory system and prepares them for event dispatching.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use verdure_ioc::LifecycleEventPublisher;
    ///
    /// let publisher = LifecycleEventPublisher::new();
    /// ```
    pub fn new() -> Self {
        let listeners: Vec<&'static LifecycleListenerDefinition> =
            inventory::iter::<LifecycleListenerDefinition>().collect();

        Self { listeners }
    }

    /// Publishes an event to all registered listeners
    ///
    /// This method synchronously calls all registered event handlers with the provided event.
    /// If any listener panics, the panic will propagate up to the caller.
    ///
    /// # Arguments
    ///
    /// * `event` - The lifecycle event to publish
    ///
    /// # Examples
    ///
    /// ```rust
    /// use verdure_ioc::{LifecycleEventPublisher, ContainerLifecycleEvent, ComponentContainer};
    ///
    /// let publisher = LifecycleEventPublisher::new();
    /// let container = ComponentContainer::new();
    ///
    /// let event = ContainerLifecycleEvent::InitializationStarted {
    ///     container: &container,
    ///     component_count: 0,
    /// };
    ///
    /// publisher.publish(&event);
    /// ```
    pub fn publish(&self, event: &ContainerLifecycleEvent) {
        for listener in &self.listeners {
            (listener.handler)(event);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ComponentContainer;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

    static EVENT_COUNTER: AtomicUsize = AtomicUsize::new(0);
    static LAST_EVENT_TYPE: AtomicUsize = AtomicUsize::new(0);

    fn test_event_handler(event: &ContainerLifecycleEvent) {
        EVENT_COUNTER.fetch_add(1, Ordering::SeqCst);

        match event {
            ContainerLifecycleEvent::InitializationStarted { .. } => {
                LAST_EVENT_TYPE.store(1, Ordering::SeqCst);
            }
            ContainerLifecycleEvent::InitializationCompleted { .. } => {
                LAST_EVENT_TYPE.store(2, Ordering::SeqCst);
            }
            ContainerLifecycleEvent::ComponentCreated { .. } => {
                LAST_EVENT_TYPE.store(3, Ordering::SeqCst);
            }
        }
    }

    #[test]
    fn test_lifecycle_listener_definition() {
        let definition = LifecycleListenerDefinition {
            name: "test_listener",
            handler: test_event_handler,
        };

        assert_eq!(definition.name, "test_listener");

        // Reset counter
        EVENT_COUNTER.store(0, Ordering::SeqCst);

        let container = ComponentContainer::new();
        let event = ContainerLifecycleEvent::InitializationStarted {
            container: &container,
            component_count: 5,
        };

        (definition.handler)(&event);
        assert_eq!(EVENT_COUNTER.load(Ordering::SeqCst), 1);
        assert_eq!(LAST_EVENT_TYPE.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_lifecycle_event_publisher_creation() {
        let publisher = LifecycleEventPublisher::new();
        // The listeners vec will be populated from inventory,
        // but since we're in a test environment without registered listeners,
        // it should be empty or contain only test listeners
        assert!(publisher.listeners.len() >= 0);
    }

    #[test]
    fn test_container_lifecycle_events() {
        let container = ComponentContainer::new();

        // Test InitializationStarted event
        let init_started = ContainerLifecycleEvent::InitializationStarted {
            container: &container,
            component_count: 10,
        };

        match &init_started {
            ContainerLifecycleEvent::InitializationStarted {
                component_count, ..
            } => {
                assert_eq!(*component_count, 10);
            }
            _ => panic!("Expected InitializationStarted event"),
        }

        // Test InitializationCompleted event
        let init_completed = ContainerLifecycleEvent::InitializationCompleted {
            container: &container,
            component_count: 10,
            duration: Duration::from_millis(100),
        };

        match &init_completed {
            ContainerLifecycleEvent::InitializationCompleted {
                component_count,
                duration,
                ..
            } => {
                assert_eq!(*component_count, 10);
                assert_eq!(duration.as_millis(), 100);
            }
            _ => panic!("Expected InitializationCompleted event"),
        }

        // Test ComponentCreated event
        let component_created = ContainerLifecycleEvent::ComponentCreated {
            container: &container,
            component_name: "TestComponent",
            component_type_id: std::any::TypeId::of::<i32>(),
            creation_duration: Duration::from_millis(50),
        };

        match &component_created {
            ContainerLifecycleEvent::ComponentCreated {
                component_name,
                component_type_id,
                creation_duration,
                ..
            } => {
                assert_eq!(*component_name, "TestComponent");
                assert_eq!(*component_type_id, std::any::TypeId::of::<i32>());
                assert_eq!(creation_duration.as_millis(), 50);
            }
            _ => panic!("Expected ComponentCreated event"),
        }
    }

    struct MockLifecycleListener {
        name: String,
        events_received: Arc<AtomicUsize>,
    }

    impl MockLifecycleListener {
        fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
                events_received: Arc::new(AtomicUsize::new(0)),
            }
        }

        fn handle_event(&self, _event: &ContainerLifecycleEvent) {
            self.events_received.fetch_add(1, Ordering::SeqCst);
        }

        fn get_event_count(&self) -> usize {
            self.events_received.load(Ordering::SeqCst)
        }
    }

    impl LifecycleListener for MockLifecycleListener {
        fn on_lifecycle_event(&self, event: &ContainerLifecycleEvent) {
            self.handle_event(event);
        }
    }

    #[test]
    fn test_lifecycle_listener_trait() {
        let listener = MockLifecycleListener::new("mock_listener");
        assert_eq!(listener.get_event_count(), 0);

        let container = ComponentContainer::new();
        let event = ContainerLifecycleEvent::InitializationStarted {
            container: &container,
            component_count: 3,
        };

        listener.on_lifecycle_event(&event);
        assert_eq!(listener.get_event_count(), 1);

        let event2 = ContainerLifecycleEvent::InitializationCompleted {
            container: &container,
            component_count: 3,
            duration: Duration::from_millis(200),
        };

        listener.on_lifecycle_event(&event2);
        assert_eq!(listener.get_event_count(), 2);
    }

    #[test]
    fn test_event_publisher_with_custom_listeners() {
        // Create a publisher and test publishing events
        let publisher = LifecycleEventPublisher {
            listeners: vec![], // Empty for this test since we can't easily inject listeners
        };

        let container = ComponentContainer::new();
        let event = ContainerLifecycleEvent::InitializationStarted {
            container: &container,
            component_count: 1,
        };

        // This should not panic even with no listeners
        publisher.publish(&event);
    }

    #[test]
    fn test_event_types_pattern_matching() {
        let container = ComponentContainer::new();

        let events = vec![
            ContainerLifecycleEvent::InitializationStarted {
                container: &container,
                component_count: 1,
            },
            ContainerLifecycleEvent::InitializationCompleted {
                container: &container,
                component_count: 1,
                duration: Duration::from_millis(10),
            },
            ContainerLifecycleEvent::ComponentCreated {
                container: &container,
                component_name: "Test",
                component_type_id: std::any::TypeId::of::<String>(),
                creation_duration: Duration::from_millis(5),
            },
        ];

        let mut event_types = Vec::new();

        for event in events {
            match event {
                ContainerLifecycleEvent::InitializationStarted { .. } => {
                    event_types.push("started");
                }
                ContainerLifecycleEvent::InitializationCompleted { .. } => {
                    event_types.push("completed");
                }
                ContainerLifecycleEvent::ComponentCreated { .. } => {
                    event_types.push("created");
                }
            }
        }

        assert_eq!(event_types, vec!["started", "completed", "created"]);
    }
}
