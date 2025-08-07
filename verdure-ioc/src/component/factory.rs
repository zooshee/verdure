use std::any::{Any, TypeId};
use std::sync::Arc;

pub trait ComponentFactory {
    fn get_component_by_type_id(&self, type_id: TypeId) -> Option<Arc<dyn Any + Send + Sync>>;

    fn get_component<T: Any + Send + Sync>(&self) -> Option<Arc<T>>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[derive(Debug, PartialEq)]
    struct TestComponent {
        value: i32,
    }

    #[derive(Debug, PartialEq)]
    struct AnotherComponent {
        name: String,
    }

    struct MockComponentFactory {
        components: HashMap<TypeId, Arc<dyn Any + Send + Sync>>,
    }

    impl MockComponentFactory {
        fn new() -> Self {
            Self {
                components: HashMap::new(),
            }
        }

        fn register<T: Any + Send + Sync>(&mut self, component: T) {
            self.components.insert(TypeId::of::<T>(), Arc::new(component));
        }
    }

    impl ComponentFactory for MockComponentFactory {
        fn get_component_by_type_id(&self, type_id: TypeId) -> Option<Arc<dyn Any + Send + Sync>> {
            self.components.get(&type_id).cloned()
        }

        fn get_component<T: Any + Send + Sync>(&self) -> Option<Arc<T>> {
            let component_any = self.get_component_by_type_id(TypeId::of::<T>())?;
            component_any.downcast().ok()
        }
    }

    #[test]
    fn test_component_factory_get_component() {
        let mut factory = MockComponentFactory::new();
        
        let test_component = TestComponent { value: 42 };
        factory.register(test_component);
        
        let retrieved: Option<Arc<TestComponent>> = factory.get_component();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().value, 42);
    }

    #[test]
    fn test_component_factory_get_component_by_type_id() {
        let mut factory = MockComponentFactory::new();
        
        let test_component = TestComponent { value: 100 };
        factory.register(test_component);
        
        let type_id = TypeId::of::<TestComponent>();
        let retrieved = factory.get_component_by_type_id(type_id);
        assert!(retrieved.is_some());
        
        let downcasted = retrieved.unwrap().downcast::<TestComponent>();
        assert!(downcasted.is_ok());
        assert_eq!(downcasted.unwrap().value, 100);
    }

    #[test]
    fn test_component_factory_multiple_components() {
        let mut factory = MockComponentFactory::new();
        
        factory.register(TestComponent { value: 123 });
        factory.register(AnotherComponent { name: "test".to_string() });
        
        let test_comp: Option<Arc<TestComponent>> = factory.get_component();
        let another_comp: Option<Arc<AnotherComponent>> = factory.get_component();
        
        assert!(test_comp.is_some());
        assert!(another_comp.is_some());
        
        assert_eq!(test_comp.unwrap().value, 123);
        assert_eq!(another_comp.unwrap().name, "test");
    }

    #[test]
    fn test_component_factory_nonexistent_component() {
        let factory = MockComponentFactory::new();
        
        let result: Option<Arc<TestComponent>> = factory.get_component();
        assert!(result.is_none());
        
        let result_by_type_id = factory.get_component_by_type_id(TypeId::of::<TestComponent>());
        assert!(result_by_type_id.is_none());
    }

    #[test]
    fn test_component_factory_downcast_failure() {
        let mut factory = MockComponentFactory::new();
        factory.register(TestComponent { value: 42 });
        
        // Try to get the TestComponent as AnotherComponent (should fail)
        let type_id = TypeId::of::<TestComponent>();
        let component_any = factory.get_component_by_type_id(type_id).unwrap();
        
        let wrong_downcast = component_any.downcast::<AnotherComponent>();
        assert!(wrong_downcast.is_err());
    }

    #[test]
    fn test_component_factory_trait_object() {
        let mut factory = MockComponentFactory::new();
        factory.register(TestComponent { value: 999 });
        
        // Test using the factory through concrete type (trait objects with generics aren't dyn compatible)
        let component: Option<Arc<TestComponent>> = factory.get_component();
        
        assert!(component.is_some());
        assert_eq!(component.unwrap().value, 999);
        
        // Test the type-erased method which is dyn compatible
        let type_id = TypeId::of::<TestComponent>();
        let component_any = factory.get_component_by_type_id(type_id);
        assert!(component_any.is_some());
        
        let downcasted = component_any.unwrap().downcast::<TestComponent>().unwrap();
        assert_eq!(downcasted.value, 999);
    }
}
