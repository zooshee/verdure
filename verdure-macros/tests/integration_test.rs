use std::sync::Arc;
use verdure_macros::Component;
use verdure::{ComponentInitializer, ComponentScope};

#[derive(Component, Debug)]
struct SimpleTestComponent {
    value: i32,
}

impl Default for SimpleTestComponent {
    fn default() -> Self {
        Self { value: 42 }
    }
}

#[derive(Component, Debug)]  
struct ComponentWithDependency {
    #[autowired]
    dependency: Arc<SimpleTestComponent>,
    name: String,
}

impl Default for ComponentWithDependency {
    fn default() -> Self {
        Self {
            dependency: Arc::new(SimpleTestComponent::default()),
            name: "default".to_string(),
        }
    }
}

#[derive(Component, Debug)]
struct ComponentWithOptionalField {
    #[autowired]
    required_dep: Arc<SimpleTestComponent>,
    optional_field: Option<String>,
    default_field: i32,
}

impl Default for ComponentWithOptionalField {
    fn default() -> Self {
        Self {
            required_dep: Arc::new(SimpleTestComponent::default()),
            optional_field: None,
            default_field: 0,
        }
    }
}

#[test]
fn test_simple_component_derive() {
    // Test that the ComponentInitializer trait is implemented correctly
    let component = SimpleTestComponent::__new(());
    // The macro-generated __new method will call Default::default() for non-autowired fields
    // For i32, Default::default() returns 0, not 42
    assert_eq!(component.value, 0);  // Default for i32 is 0
    
    // Test that the default scope is Singleton
    match SimpleTestComponent::__scope() {
        ComponentScope::Singleton => assert!(true),
        ComponentScope::Prototype => panic!("Expected Singleton scope"),
    }
}

#[test]
fn test_component_with_dependency() {
    let simple_comp = Arc::new(SimpleTestComponent { value: 100 });
    let deps = (simple_comp.clone(),);
    
    let component = ComponentWithDependency::__new(deps);
    assert_eq!(component.dependency.value, 100);
    // The macro initializes non-autowired fields using Default::default()
    // Since name is not annotated with #[autowired], it should get Default::default() for String
    // which is an empty string, not "default"
    assert_eq!(component.name, "");
}

#[test]
fn test_component_with_optional_field() {
    let simple_comp = Arc::new(SimpleTestComponent { value: 200 });
    let deps = (simple_comp.clone(),);
    
    let component = ComponentWithOptionalField::__new(deps);
    assert_eq!(component.required_dep.value, 200);
    assert!(component.optional_field.is_none());
    assert_eq!(component.default_field, 0);
}

#[test]
fn test_component_debug_trait() {
    let simple = SimpleTestComponent { value: 999 };
    let debug_str = format!("{:?}", simple);
    assert!(debug_str.contains("SimpleTestComponent"));
    assert!(debug_str.contains("999"));
}

#[test]
fn test_component_initializer_trait_bounds() {
    // Test that ComponentInitializer is properly implemented
    fn requires_component_initializer<T: ComponentInitializer>() {}
    
    requires_component_initializer::<SimpleTestComponent>();
    requires_component_initializer::<ComponentWithDependency>();
    requires_component_initializer::<ComponentWithOptionalField>();
}

#[test]
fn test_arc_dependency_type() {
    // Test that the macro correctly handles Arc<T> dependencies
    let simple_comp = Arc::new(SimpleTestComponent { value: 300 });
    let deps = (simple_comp.clone(),);
    
    let component = ComponentWithDependency::__new(deps);
    
    // The dependency should be the same Arc instance
    assert!(Arc::ptr_eq(&component.dependency, &simple_comp));
}

// Test error conditions with compile-time failures would need to be in separate files
// or use trybuild crate for negative testing

#[test]
fn test_multiple_dependencies() {
    // This would test a component with multiple dependencies
    // For now, let's test that single dependency works correctly
    
    #[derive(Component, Debug)]
    struct MultiDepComponent {
        #[autowired]
        dep1: Arc<SimpleTestComponent>,
        field1: String,
    }
    
    let simple_comp = Arc::new(SimpleTestComponent { value: 500 });
    let deps = (simple_comp.clone(),);
    
    let component = MultiDepComponent::__new(deps);
    assert_eq!(component.dep1.value, 500);
    // field1 will be initialized with Default::default() which is "" for String
    assert_eq!(component.field1, "");
}

#[test]
fn test_component_scope_singleton() {
    #[derive(Component, Debug)]
    struct SingletonComponent {
        value: u32,
    }
    
    impl Default for SingletonComponent {
        fn default() -> Self {
            Self { value: 123 }
        }
    }
    
    match SingletonComponent::__scope() {
        ComponentScope::Singleton => assert!(true),
        ComponentScope::Prototype => panic!("Expected Singleton scope"),
    }
}