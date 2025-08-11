//! Component factory trait and implementations
//!
//! This module defines the `ComponentFactory` trait which provides methods for
//! retrieving components from the IoC container with type safety.

use std::any::{Any, TypeId};
use std::sync::Arc;

/// Trait for retrieving components from a container
///
/// `ComponentFactory` provides both type-erased and type-safe methods for
/// retrieving component instances from the container. This trait is implemented
/// by `ComponentContainer` to provide a clean API for component retrieval.
///
/// # Examples
///
/// ```rust
/// use verdure_ioc::{ComponentFactory, ComponentContainer};
/// use std::sync::Arc;
///
/// #[derive(Debug)]
/// struct MyService {
///     value: i32,
/// }
///
/// let container = ComponentContainer::new();
/// container.register_component(Arc::new(MyService { value: 42 }));
///
/// // Type-safe retrieval
/// let service: Option<Arc<MyService>> = container.get_component();
/// assert!(service.is_some());
///
/// // Type-erased retrieval
/// let service_any = container.get_component_by_type_id(std::any::TypeId::of::<MyService>());
/// assert!(service_any.is_some());
/// ```
pub trait ComponentFactory {
    /// Retrieves a component instance by its TypeId
    ///
    /// This method returns the component as a type-erased `Arc<dyn Any + Send + Sync>`,
    /// which can be downcast to the specific type if needed.
    ///
    /// # Arguments
    ///
    /// * `type_id` - The TypeId of the component to retrieve
    ///
    /// # Returns
    ///
    /// `Some(Arc<dyn Any + Send + Sync>)` if the component exists, `None` otherwise
    fn get_component_by_type_id(&self, type_id: TypeId) -> Option<Arc<dyn Any + Send + Sync>>;

    /// Retrieves a component instance with compile-time type safety
    ///
    /// This method provides a type-safe way to retrieve components from the container.
    /// The compiler will ensure that the returned type matches the requested type.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The type of component to retrieve. Must implement `Any + Send + Sync`
    ///
    /// # Returns
    ///
    /// `Some(Arc<T>)` if the component exists and can be downcast to type `T`, `None` otherwise
    ///
    /// # Examples
    ///
    /// ```rust
    /// use verdure_ioc::{ComponentFactory, ComponentContainer};
    /// use std::sync::Arc;
    ///
    /// #[derive(Debug)]
    /// struct DatabaseService {
    ///     url: String,
    /// }
    ///
    /// let container = ComponentContainer::new();
    /// let db_service = Arc::new(DatabaseService { url: "localhost:5432".to_string() });
    /// container.register_component(db_service);
    ///
    /// // Retrieve with type safety
    /// let retrieved: Option<Arc<DatabaseService>> = container.get_component();
    /// match retrieved {
    ///     Some(service) => println!("Database URL: {}", service.url),
    ///     None => println!("DatabaseService not found"),
    /// }
    /// ```
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
            self.components
                .insert(TypeId::of::<T>(), Arc::new(component));
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
        factory.register(AnotherComponent {
            name: "test".to_string(),
        });

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
