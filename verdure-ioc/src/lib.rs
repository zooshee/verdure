mod component;
mod container;
mod event;

pub use component::{
    ComponentInitializer, ComponentDefinition, ComponentInstance, ComponentScope, factory::ComponentFactory,
};

pub use container::ComponentContainer;

pub use event::{ContainerLifecycleEvent, LifecycleListener, LifecycleEventPublisher, LifecycleListenerDefinition};



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
