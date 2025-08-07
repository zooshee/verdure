//! Component definition and initialization system
//! 
//! This module provides the core abstractions for defining and initializing components
//! in the IoC container. It includes component scopes, dependency management, and
//! component definition structures.

pub mod factory;

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;
use verdure_core::error::component::ComponentError;

/// Type alias for component instances stored in the container
/// 
/// All components are stored as `Arc<dyn Any + Send + Sync>` to enable
/// type-safe downcasting while maintaining thread safety.
pub type ComponentInstance = Arc<dyn Any + Send + Sync>;

/// Enumeration of component lifecycle scopes
/// 
/// This enum defines how component instances are managed by the container.
/// 
/// # Variants
/// 
/// * `Singleton` - Only one instance of the component exists throughout the application lifecycle
/// * `Prototype` - A new instance is created each time the component is requested
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentScope {
    /// Single shared instance across the entire application
    Singleton,
    /// New instance created on each request
    Prototype,
}

/// Definition structure for registering components with the container
/// 
/// `ComponentDefinition` contains all the metadata and factory functions needed
/// to create and manage component instances. This structure is typically generated
/// by the `#[derive(Component)]` macro but can also be created manually.
/// 
/// # Examples
/// 
/// ```rust
/// use verdure_ioc::{ComponentDefinition, ComponentScope, ComponentInstance};
/// use std::collections::HashMap;
/// use std::sync::Arc;
/// use std::any::TypeId;
/// 
/// #[derive(Debug)]
/// struct MyService {
///     value: i32,
/// }
/// 
/// let definition = ComponentDefinition {
///     type_id: || TypeId::of::<MyService>(),
///     type_name: "MyService",
///     scope: || ComponentScope::Singleton,
///     dependencies: || vec![],
///     creator: |_deps| Ok(Arc::new(MyService { value: 42 })),
/// };
/// ```
#[derive(Debug)]
pub struct ComponentDefinition {
    /// Function that returns the TypeId of the component
    pub type_id: fn() -> TypeId,
    /// Human-readable name of the component type
    pub type_name: &'static str,
    /// Function that returns the component's scope
    pub scope: fn() -> ComponentScope,
    /// Function that returns the TypeIds of the component's dependencies
    pub dependencies: fn() -> Vec<TypeId>,
    /// Function that creates an instance of the component given its dependencies
    pub creator: fn(deps: HashMap<TypeId, ComponentInstance>) -> Result<ComponentInstance, ComponentError>,
}

inventory::collect!(ComponentDefinition);

/// Trait for components that can be automatically initialized by the container
/// 
/// This trait is typically implemented by the `#[derive(Component)]` macro,
/// but can also be implemented manually for custom component initialization logic.
/// 
/// # Type Parameters
/// 
/// * `Dependencies` - A tuple type representing the component's dependencies
/// 
/// # Examples
/// 
/// ```rust
/// use verdure_ioc::{ComponentInitializer, ComponentScope};
/// use std::sync::Arc;
/// 
/// struct DatabaseService {
///     connection_string: String,
/// }
/// 
/// struct UserService {
///     db: Arc<DatabaseService>,
/// }
/// 
/// impl ComponentInitializer for UserService {
///     type Dependencies = (Arc<DatabaseService>,);
/// 
///     fn __new(deps: Self::Dependencies) -> Self {
///         let (db,) = deps;
///         Self { db }
///     }
/// 
///     fn __scope() -> ComponentScope {
///         ComponentScope::Singleton
///     }
/// }
/// ```
pub trait ComponentInitializer: Sized {
    /// The type representing this component's dependencies
    type Dependencies;
    
    /// Creates a new instance of the component with the provided dependencies
    /// 
    /// # Arguments
    /// 
    /// * `deps` - The resolved dependencies for this component
    fn __new(deps: Self::Dependencies) -> Self;
    
    /// Returns the scope for this component type
    fn __scope() -> ComponentScope;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    struct SimpleComponent {
        value: i32,
    }

    #[derive(Debug)]
    struct ComponentWithDependencies {
        simple: Arc<SimpleComponent>,
        message: String,
    }

    impl ComponentInitializer for SimpleComponent {
        type Dependencies = ();

        fn __new(_deps: Self::Dependencies) -> Self {
            SimpleComponent { value: 42 }
        }

        fn __scope() -> ComponentScope {
            ComponentScope::Singleton
        }
    }

    impl ComponentInitializer for ComponentWithDependencies {
        type Dependencies = (Arc<SimpleComponent>,);

        fn __new(deps: Self::Dependencies) -> Self {
            let (simple,) = deps;
            ComponentWithDependencies {
                simple,
                message: "Hello".to_string(),
            }
        }

        fn __scope() -> ComponentScope {
            ComponentScope::Prototype
        }
    }

    #[test]
    fn test_component_scope() {
        match ComponentScope::Singleton {
            ComponentScope::Singleton => assert!(true),
            ComponentScope::Prototype => assert!(false),
        }

        match ComponentScope::Prototype {
            ComponentScope::Singleton => assert!(false),
            ComponentScope::Prototype => assert!(true),
        }
    }

    #[test]
    fn test_simple_component_initializer() {
        let component = SimpleComponent::__new(());
        assert_eq!(component.value, 42);
        
        match SimpleComponent::__scope() {
            ComponentScope::Singleton => assert!(true),
            ComponentScope::Prototype => assert!(false),
        }
    }

    #[test]
    fn test_component_with_dependencies_initializer() {
        let simple_component = Arc::new(SimpleComponent { value: 100 });
        let deps = (simple_component.clone(),);
        
        let component = ComponentWithDependencies::__new(deps);
        assert_eq!(component.simple.value, 100);
        assert_eq!(component.message, "Hello");
        
        match ComponentWithDependencies::__scope() {
            ComponentScope::Singleton => assert!(false),
            ComponentScope::Prototype => assert!(true),
        }
    }

    #[test]
    fn test_component_definition_structure() {
        let type_id_fn = || std::any::TypeId::of::<SimpleComponent>();
        let type_name = "SimpleComponent";
        let scope_fn = || ComponentScope::Singleton;
        let dependencies_fn = || vec![];
        let creator_fn = |deps: HashMap<TypeId, ComponentInstance>| {
            assert!(deps.is_empty());
            let instance = SimpleComponent::__new(());
            Ok(Arc::new(instance) as ComponentInstance)
        };

        let definition = ComponentDefinition {
            type_id: type_id_fn,
            type_name,
            scope: scope_fn,
            dependencies: dependencies_fn,
            creator: creator_fn,
        };

        assert_eq!((definition.type_id)(), TypeId::of::<SimpleComponent>());
        assert_eq!(definition.type_name, "SimpleComponent");
        assert!(matches!((definition.scope)(), ComponentScope::Singleton));
        assert!((definition.dependencies)().is_empty());
        
        let result = (definition.creator)(HashMap::new());
        assert!(result.is_ok());
    }

    #[test]
    fn test_component_instance_type() {
        let simple_component = SimpleComponent { value: 123 };
        let instance: ComponentInstance = Arc::new(simple_component);
        
        // Test that we can downcast back to the original type
        let downcasted = instance.downcast::<SimpleComponent>();
        assert!(downcasted.is_ok());
        assert_eq!(downcasted.unwrap().value, 123);
    }

    #[test]
    fn test_component_definition_with_dependencies() {
        let type_id_fn = || std::any::TypeId::of::<ComponentWithDependencies>();
        let type_name = "ComponentWithDependencies";
        let scope_fn = || ComponentScope::Prototype;
        let dependencies_fn = || vec![TypeId::of::<SimpleComponent>()];
        let creator_fn = |deps: HashMap<TypeId, ComponentInstance>| {
            let simple_dep = deps.get(&TypeId::of::<SimpleComponent>())
                .ok_or_else(|| verdure_core::error::component::ComponentError::DependencyNotFound("SimpleComponent".to_string()))?
                .clone()
                .downcast::<SimpleComponent>()
                .map_err(|_| verdure_core::error::component::ComponentError::DowncastFailed("SimpleComponent".to_string()))?;
                
            let instance = ComponentWithDependencies::__new((simple_dep,));
            Ok(Arc::new(instance) as ComponentInstance)
        };

        let definition = ComponentDefinition {
            type_id: type_id_fn,
            type_name,
            scope: scope_fn,
            dependencies: dependencies_fn,
            creator: creator_fn,
        };

        assert_eq!((definition.type_id)(), TypeId::of::<ComponentWithDependencies>());
        assert_eq!(definition.type_name, "ComponentWithDependencies");
        assert!(matches!((definition.scope)(), ComponentScope::Prototype));
        assert_eq!((definition.dependencies)(), vec![TypeId::of::<SimpleComponent>()]);
        
        // Test creator with proper dependency
        let mut deps = HashMap::new();
        let simple_instance: ComponentInstance = Arc::new(SimpleComponent { value: 999 });
        deps.insert(TypeId::of::<SimpleComponent>(), simple_instance);
        
        let result = (definition.creator)(deps);
        assert!(result.is_ok());
        
        let component_instance = result.unwrap();
        let downcasted = component_instance.downcast::<ComponentWithDependencies>().unwrap();
        assert_eq!(downcasted.simple.value, 999);
        assert_eq!(downcasted.message, "Hello");
    }

    #[test]
    fn test_component_definition_creator_missing_dependency() {
        let creator_fn = |deps: HashMap<TypeId, ComponentInstance>| {
            let _simple_dep = deps.get(&TypeId::of::<SimpleComponent>())
                .ok_or_else(|| verdure_core::error::component::ComponentError::DependencyNotFound("SimpleComponent".to_string()))?;
            Ok(Arc::new(ComponentWithDependencies {
                simple: Arc::new(SimpleComponent { value: 0 }),
                message: "test".to_string(),
            }) as ComponentInstance)
        };

        let result = creator_fn(HashMap::new());
        assert!(result.is_err());
        
        if let Err(error) = result {
            assert!(matches!(error, verdure_core::error::component::ComponentError::DependencyNotFound(_)));
        }
    }
}
