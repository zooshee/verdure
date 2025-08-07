pub mod factory;

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;
use verdure_core::error::component::ComponentError;

pub type ComponentInstance = Arc<dyn Any + Send + Sync>;

pub enum ComponentScope {
    Singleton,
    Prototype,
}

#[derive(Debug)]
pub struct ComponentDefinition {
    pub type_id: fn() -> TypeId,
    pub type_name: &'static str,
    pub scope: fn() -> ComponentScope,
    pub dependencies: fn() -> Vec<TypeId>,
    pub creator: fn(deps: HashMap<TypeId, ComponentInstance>) -> Result<ComponentInstance, ComponentError>,
}

inventory::collect!(ComponentDefinition);

pub trait ComponentInitializer: Sized {
    type Dependencies;
    fn __new(deps: Self::Dependencies) -> Self;
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
