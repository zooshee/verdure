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

