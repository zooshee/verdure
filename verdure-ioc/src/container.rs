//! IoC container implementation for the Verdure ecosystem
//! 
//! This module provides the core `ComponentContainer` implementation that serves as the
//! foundation for dependency injection across the entire Verdure ecosystem. It manages
//! component lifecycles, resolves dependencies, and provides the runtime infrastructure
//! that enables Verdure's declarative programming model.

use crate::{ComponentDefinition, ComponentFactory, ComponentInstance, ComponentScope};
use dashmap::{DashMap, DashSet};
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use verdure_core::error::container::ContainerError;
use crate::event::{ContainerLifecycleEvent, LifecycleEventPublisher};

/// Component descriptor for identifying components in the container
/// 
/// `ComponentDescriptor` uniquely identifies components within the container
/// using both their type and an optional qualifier string.
/// 
/// # Examples
/// 
/// ```rust
/// use verdure_ioc::ComponentContainer;
/// use std::any::TypeId;
/// 
/// #[derive(Debug)]
/// struct DatabaseService;
/// 
/// let container = ComponentContainer::new();
/// // ComponentDescriptor is used internally by the container
/// // Users typically don't need to create descriptors manually
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ComponentDescriptor {
    /// The TypeId of the component
    pub type_id: TypeId,
    /// Optional qualifier for named instances
    pub qualifier: Option<&'static str>,
}

impl ComponentDescriptor {
    /// Creates a new ComponentDescriptor with the given type and optional qualifier
    /// 
    /// # Arguments
    /// 
    /// * `type_id` - The TypeId of the component
    /// * `qualifier` - Optional qualifier string for named instances
    pub fn new(type_id: TypeId, qualifier: Option<&'static str>) -> Self {
        Self { type_id, qualifier }
    }
    
    /// Creates a ComponentDescriptor for the specified type without a qualifier
    /// 
    /// # Type Parameters
    /// 
    /// * `T` - The component type
    pub fn for_type<T: 'static>() -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            qualifier: None,
        }
    }
    
    /// Creates a ComponentDescriptor for the specified type with a qualifier
    /// 
    /// # Arguments
    /// 
    /// * `qualifier` - The qualifier string for named instances
    /// 
    /// # Type Parameters
    /// 
    /// * `T` - The component type
    pub fn with_qualifier<T: 'static>(qualifier: &'static str) -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            qualifier: Some(qualifier),
        }
    }
}

/// Statistics for component creation and access
/// 
/// `ComponentStats` tracks various metrics about component usage for
/// monitoring and debugging purposes.
#[derive(Debug, Default, Clone)]
pub struct ComponentStats {
    /// When the component was first created
    pub created_at: Option<Instant>,
    /// When the component was last accessed
    pub last_accessed: Option<Instant>,
    /// Total number of times the component has been accessed
    pub access_count: u64,
    /// Time taken to create the component instance (in milliseconds)
    pub creation_time: u64,
}

/// The central IoC container for the Verdure ecosystem
/// 
/// `ComponentContainer` serves as the heart of the Verdure ecosystem's dependency injection system.
/// It provides the runtime infrastructure that enables all Verdure modules - from web frameworks
/// to data access layers - to work together through declarative component management.
/// 
/// This container powers the entire Verdure ecosystem by providing:
/// * **Cross-module Integration**: Components from different Verdure modules can seamlessly depend on each other
/// * **Ecosystem Coherence**: Unified component management across verdure-web, verdure-data, verdure-security, etc.
/// 
/// # Features
/// 
/// * **Thread-safe**: Uses concurrent data structures for safe multi-threaded access
/// * **Dependency Resolution**: Automatically resolves and injects component dependencies
/// * **Circular Dependency Detection**: Prevents infinite dependency loops during resolution
/// * **Lifecycle Events**: Publishes events during container and component lifecycle operations
/// * **Statistics Tracking**: Maintains creation and access statistics for monitoring
/// 
/// # Examples
/// 
/// ```rust
/// use verdure_ioc::{ComponentContainer, ComponentFactory};
/// use std::sync::Arc;
/// 
/// #[derive(Debug)]
/// struct DatabaseService {
///     connection_string: String,
/// }
/// 
/// // Create container
/// let container = ComponentContainer::new();
/// 
/// // Register a component
/// let db_service = Arc::new(DatabaseService {
///     connection_string: "postgres://localhost:5432/mydb".to_string(),
/// });
/// container.register_component(db_service);
/// 
/// // Retrieve the component
/// let retrieved: Option<Arc<DatabaseService>> = container.get_component();
/// assert!(retrieved.is_some());
/// 
/// // Initialize automatic components (from #[derive(Component)])
/// let result = container.initialize();
/// assert!(result.is_ok());
/// ```
pub struct ComponentContainer {
    /// Map of component descriptors to their instances
    components: DashMap<ComponentDescriptor, ComponentInstance>,
    /// Set tracking which components are currently being initialized (for circular dependency detection)
    initializing: DashSet<TypeId>,
    /// Statistics for each component
    stats: DashMap<ComponentDescriptor, ComponentStats>,
    /// Event publisher for lifecycle events
    lifecycle_publisher: Arc<LifecycleEventPublisher>,
}

impl ComponentContainer {
    /// Creates a new empty ComponentContainer
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use verdure_ioc::ComponentContainer;
    /// 
    /// let container = ComponentContainer::new();
    /// ```
    pub fn new() -> Self {
        Self {
            components: DashMap::new(),
            initializing: DashSet::new(),
            stats: DashMap::new(),
            lifecycle_publisher: Arc::new(LifecycleEventPublisher::new()),
        }
    }

    /// Initializes the container by discovering and creating all registered components
    /// 
    /// This method scans for all components registered via the `#[derive(Component)]` macro
    /// and creates instances of them, resolving their dependencies automatically.
    /// It also publishes lifecycle events during the initialization process.
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - If initialization completed successfully
    /// * `Err(ContainerError)` - If there was an error during initialization (e.g., circular dependencies)
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use verdure_ioc::ComponentContainer;
    /// 
    /// let container = ComponentContainer::new();
    /// match container.initialize() {
    ///     Ok(_) => println!("Container initialized successfully"),
    ///     Err(e) => eprintln!("Initialization failed: {}", e),
    /// }
    /// ```
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

    /// Registers a pre-created component instance with the container
    /// 
    /// This method allows manual registration of component instances that have been
    /// created outside the container's automatic initialization process.
    /// 
    /// # Arguments
    /// 
    /// * `instance` - The component instance to register
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use verdure_ioc::ComponentContainer;
    /// use std::sync::Arc;
    /// 
    /// #[derive(Debug)]
    /// struct ConfigService {
    ///     config_path: String,
    /// }
    /// 
    /// let container = ComponentContainer::new();
    /// let config = Arc::new(ConfigService {
    ///     config_path: "/etc/myapp/config.toml".to_string(),
    /// });
    /// 
    /// container.register_component(config);
    /// ```
    pub fn register_component(&self, instance: ComponentInstance) {
        let type_id = (*instance).type_id();
        self.register_component_by_type_id(type_id, instance);
    }

    /// Registers a component instance with the container using a specific TypeId
    /// 
    /// This method is useful when you need to register a component with a different
    /// type identity than its concrete type.
    /// 
    /// # Arguments
    /// 
    /// * `type_id` - The TypeId to use for component registration
    /// * `instance` - The component instance to register
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
