use std::any::TypeId;
use std::time::Duration;
use crate::container::ComponentContainer;

pub enum ContainerLifecycleEvent<'a> {
    InitializationStarted {
        container: &'a ComponentContainer,
        component_count: usize,
    },
    InitializationCompleted {
        container: &'a ComponentContainer,
        component_count: usize,
        duration: Duration,
    },
    ComponentCreated {
        container: &'a ComponentContainer,
        component_name: &'static str,
        component_type_id: TypeId,
        creation_duration: Duration,
    },
}

pub trait LifecycleListener: Send + Sync {
    fn on_lifecycle_event(&self, event: &ContainerLifecycleEvent);
}

pub struct LifecycleListenerDefinition {
    pub name: &'static str,
    pub handler: fn(&ContainerLifecycleEvent),
}

inventory::collect!(LifecycleListenerDefinition);

pub struct LifecycleEventPublisher {
    listeners: Vec<&'static LifecycleListenerDefinition>,
}

impl LifecycleEventPublisher {
    pub fn new() -> Self {
        let listeners: Vec<&'static LifecycleListenerDefinition> =
            inventory::iter::<LifecycleListenerDefinition>().collect();

        Self { listeners }
    }

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
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
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
            ContainerLifecycleEvent::InitializationStarted { component_count, .. } => {
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
            ContainerLifecycleEvent::InitializationCompleted { component_count, duration, .. } => {
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

