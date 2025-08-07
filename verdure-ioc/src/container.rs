use crate::{ComponentDefinition, ComponentFactory, ComponentInstance, ComponentScope};
use dashmap::{DashMap, DashSet};
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use verdure_core::error::container::ContainerError;
use crate::event::{ContainerLifecycleEvent, LifecycleEventPublisher};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ComponentDescriptor {
    pub type_id: TypeId,
    pub qualifier: Option<&'static str>,
}

impl ComponentDescriptor {
    pub fn new(type_id: TypeId, qualifier: Option<&'static str>) -> Self {
        Self { type_id, qualifier }
    }
    pub fn for_type<T: 'static>() -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            qualifier: None,
        }
    }
    pub fn with_qualifier<T: 'static>(qualifier: &'static str) -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            qualifier: Some(qualifier),
        }
    }
}
#[derive(Debug, Default, Clone)]
pub struct ComponentStats {
    pub created_at: Option<Instant>,
    pub last_accessed: Option<Instant>,
    pub access_count: u64,
    pub creation_time: u64,
}
pub struct ComponentContainer {
    components: DashMap<ComponentDescriptor, ComponentInstance>,
    initializing: DashSet<TypeId>,
    stats: DashMap<ComponentDescriptor, ComponentStats>,
    lifecycle_publisher: Arc<LifecycleEventPublisher>,
}

impl ComponentContainer {
    pub fn new() -> Self {
        Self {
            components: DashMap::new(),
            initializing: DashSet::new(),
            stats: DashMap::new(),
            lifecycle_publisher: Arc::new(LifecycleEventPublisher::new()),
        }
    }

    pub fn initialize(&self) -> Result<(), ContainerError> {

        let component_count = inventory::iter::<ComponentDefinition>().count();

        self.lifecycle_publisher.publish(&ContainerLifecycleEvent::InitializationStarted {
            container: self,
            component_count,
        });

        let start_time = Instant::now();

        let mut def_map = HashMap::new();
        for def in inventory::iter::<ComponentDefinition> {
            let type_id = (def.type_id)();
            def_map.insert(type_id, def);
        }

        for def in inventory::iter::<ComponentDefinition> {
            let type_id = (def.type_id)();
            let descriptor = ComponentDescriptor::new(type_id, None);

            if !self.components.contains_key(&descriptor) {
                self.resolve_bean(&descriptor, &def_map)?;
            } else {
                // TODO: Duplicate registration failed
            }
        }

        self.lifecycle_publisher.publish(&ContainerLifecycleEvent::InitializationCompleted {
            container: self,
            component_count: self.components.len(),
            duration: start_time.elapsed(),
        });


        Ok(())
    }

    pub fn register_component(&self, instance: ComponentInstance) {
        let type_id = (*instance).type_id();
        self.register_component_by_type_id(type_id, instance);
    }

    pub fn register_component_by_type_id(&self, type_id: TypeId, instance: ComponentInstance) {
        let descriptor = ComponentDescriptor::new(type_id, None);
        self.components.insert(descriptor, instance);
    }

    fn resolve_bean(
        &self,
        descriptor: &ComponentDescriptor,
        def_map: &HashMap<TypeId, &ComponentDefinition>,
    ) -> Result<ComponentInstance, ContainerError> {
        if !self.initializing.insert(descriptor.type_id) {
            let type_name = def_map
                .get(&descriptor.type_id)
                .map_or("Unknow", |d| d.type_name);
            return Err(ContainerError::circular_dependency(format!(
                "{}",
                type_name
            )));
        }

        let def = match def_map.get(&descriptor.type_id) {
            Some(d) => *d,
            None => {
                self.initializing.remove(&descriptor.type_id);
                return Err(ContainerError::not_found(format!(
                    "Bean definition not found for type ID {:?}",
                    descriptor.type_id
                )));
            }
        };

        let dependencies = (def.dependencies)();
        let mut deps_map = HashMap::new();
        for dep_id in dependencies {
            let dep_descriptor = ComponentDescriptor::new(dep_id, None);

            // exist in components
            if let Some(instance) = self.components.get(&dep_descriptor) {
                deps_map.insert(dep_id, instance.clone());
                continue;
            }

            if let Some(_dep_def) = def_map.get(&dep_id) {
                let dep_instance = self.resolve_bean(&dep_descriptor, def_map)?;
                deps_map.insert(dep_id, dep_instance);
            } else {
                self.initializing.remove(&descriptor.type_id);
                return Err(ContainerError::not_found(format!(
                    "Dependency not found for type ID {:?}",
                    dep_id
                )));
            }
        }

        let start = Instant::now();
        let instance = match (def.creator)(deps_map) {
            Ok(i) => i,
            Err(e) => {
                self.initializing.remove(&descriptor.type_id);
                return Err(ContainerError::creation_failed(format!(
                    "Failed to create bean '{}': '{}'",
                    def.type_name, e
                )));
            }
        };
        let creation_time = start.elapsed();

        self.lifecycle_publisher.publish(&ContainerLifecycleEvent::ComponentCreated {
            container: self,
            component_name: def.type_name,
            component_type_id: descriptor.type_id,
            creation_duration: creation_time,
        });


        self.initializing.remove(&descriptor.type_id);

        match (def.scope)() {
            ComponentScope::Singleton => {
                self.components.insert(descriptor.clone(), instance.clone());
            },
            _ => {}
        };

        self.stats.insert(
            descriptor.clone(),
            ComponentStats {
                created_at: Some(Instant::now()),
                last_accessed: Some(Instant::now()),
                access_count: 1,
                creation_time: creation_time.as_millis() as u64,
            },
        );

        Ok(instance)
    }
}

impl ComponentFactory for ComponentContainer {
    fn get_component_by_type_id(&self, type_id: TypeId) -> Option<Arc<dyn Any + Send + Sync>> {
        Some(
            self.components
                .get(&ComponentDescriptor::new(type_id, None))?
                .clone(),
        )
    }

    fn get_component<T: Any + Send + Sync>(&self) -> Option<Arc<T>> {
        let component_any = self.get_component_by_type_id(TypeId::of::<T>())?;
        component_any.downcast().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ComponentInitializer;
    use std::sync::atomic::{AtomicU32, Ordering};

    #[derive(Debug)]
    struct TestComponent {
        value: u32,
    }

    impl TestComponent {
        fn new(value: u32) -> Self {
            Self { value }
        }
    }

    #[derive(Debug)]
    struct TestComponentWithDeps {
        dependency: Arc<TestComponent>,
        value: String,
    }

    static CREATION_COUNTER: AtomicU32 = AtomicU32::new(0);

    impl ComponentInitializer for TestComponent {
        type Dependencies = ();

        fn __new(_deps: Self::Dependencies) -> Self {
            CREATION_COUNTER.fetch_add(1, Ordering::SeqCst);
            TestComponent::new(42)
        }

        fn __scope() -> crate::ComponentScope {
            crate::ComponentScope::Singleton
        }
    }

    impl ComponentInitializer for TestComponentWithDeps {
        type Dependencies = (Arc<TestComponent>,);

        fn __new(deps: Self::Dependencies) -> Self {
            let (dependency,) = deps;
            TestComponentWithDeps {
                dependency,
                value: "test".to_string(),
            }
        }

        fn __scope() -> crate::ComponentScope {
            crate::ComponentScope::Singleton
        }
    }

    #[test]
    fn test_component_descriptor() {
        let desc1 = ComponentDescriptor::for_type::<TestComponent>();
        let desc2 = ComponentDescriptor::new(TypeId::of::<TestComponent>(), None);
        assert_eq!(desc1, desc2);

        let desc_with_qualifier = ComponentDescriptor::with_qualifier::<TestComponent>("test");
        assert_ne!(desc1, desc_with_qualifier);
        assert_eq!(desc_with_qualifier.qualifier, Some("test"));
    }

    #[test]
    fn test_container_creation() {
        let container = ComponentContainer::new();
        assert!(container.components.is_empty());
        assert!(container.initializing.is_empty());
        assert!(container.stats.is_empty());
    }

    #[test]
    fn test_manual_component_registration() {
        let container = ComponentContainer::new();
        let test_component = Arc::new(TestComponent::new(100));
        
        container.register_component(test_component.clone());
        
        let retrieved: Option<Arc<TestComponent>> = container.get_component();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().value, 100);
    }

    #[test]
    fn test_register_component_by_type_id() {
        let container = ComponentContainer::new();
        let test_component = Arc::new(TestComponent::new(200));
        let type_id = TypeId::of::<TestComponent>();
        
        container.register_component_by_type_id(type_id, test_component);
        
        let retrieved = container.get_component_by_type_id(type_id);
        assert!(retrieved.is_some());
        
        let downcast_component: Result<Arc<TestComponent>, _> = retrieved.unwrap().downcast();
        assert!(downcast_component.is_ok());
        assert_eq!(downcast_component.unwrap().value, 200);
    }

    #[test]
    fn test_get_nonexistent_component() {
        let container = ComponentContainer::new();
        let result: Option<Arc<TestComponent>> = container.get_component();
        assert!(result.is_none());
    }

    #[test]
    fn test_component_stats_default() {
        let stats = ComponentStats::default();
        assert!(stats.created_at.is_none());
        assert!(stats.last_accessed.is_none());
        assert_eq!(stats.access_count, 0);
        assert_eq!(stats.creation_time, 0);
    }

    #[test]
    fn test_component_descriptor_hash_and_eq() {
        let desc1 = ComponentDescriptor::for_type::<TestComponent>();
        let desc2 = ComponentDescriptor::for_type::<TestComponent>();
        let desc3 = ComponentDescriptor::with_qualifier::<TestComponent>("test");

        assert_eq!(desc1, desc2);
        assert_ne!(desc1, desc3);

        // Test hash by inserting into HashMap
        let mut map = std::collections::HashMap::new();
        map.insert(desc1.clone(), "value1");
        map.insert(desc3.clone(), "value2");

        assert_eq!(map.get(&desc2), Some(&"value1"));
        assert_eq!(map.get(&desc3), Some(&"value2"));
    }
}
