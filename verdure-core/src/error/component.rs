use crate::error::container::{ContainerError, ContainerErrorKind};
use std::fmt;

#[derive(Debug)]
pub enum ComponentError {
    DependencyNotFound(String),
    DowncastFailed(String),
    CircularDependency(String),
    ConfigurationError(String),
    CreationError(String),
    NotFound(String),
}

impl fmt::Display for ComponentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ComponentError::DependencyNotFound(s) => write!(f, "Dependency '{}' not found", s),
            ComponentError::DowncastFailed(s) => write!(f, "Failed to downcast dependency '{}'", s),
            ComponentError::CircularDependency(s) => {
                write!(f, "Circular dependency detected: {}", s)
            }
            ComponentError::ConfigurationError(s) => write!(f, "Configuration error: {}", s),
            ComponentError::CreationError(s) => write!(f, "Component creation error: {}", s),
            ComponentError::NotFound(s) => write!(f, "Component not found: {}", s),
        }
    }
}

impl From<ContainerError> for ComponentError {
    fn from(err: ContainerError) -> Self {
        match err.kind {
            ContainerErrorKind::NotFound => ComponentError::NotFound(err.message),
            ContainerErrorKind::CircularDependency => {
                ComponentError::CircularDependency(err.message)
            }
            ContainerErrorKind::CreationFailed => ComponentError::CreationError(err.message),
            ContainerErrorKind::TypeCastFailed => ComponentError::DowncastFailed(err.message),
            ContainerErrorKind::Configuration => ComponentError::ConfigurationError(err.message),
            ContainerErrorKind::Other => ComponentError::CreationError(err.message),
        }
    }
}

impl From<ComponentError> for ContainerError {
    fn from(err: ComponentError) -> Self {
        match err {
            ComponentError::NotFound(msg) => ContainerError::not_found(msg),
            ComponentError::CircularDependency(msg) => ContainerError::circular_dependency(msg),
            ComponentError::CreationError(msg) => ContainerError::creation_failed(msg),
            ComponentError::DowncastFailed(msg) => ContainerError::type_cast_failed(msg),
            ComponentError::ConfigurationError(msg) => ContainerError::configuration(msg),
            ComponentError::DependencyNotFound(msg) => {
                ContainerError::not_found(format!("Dependency: {}", msg))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_error_display() {
        let errors = vec![
            ComponentError::DependencyNotFound("TestDep".to_string()),
            ComponentError::DowncastFailed("TestComponent".to_string()),
            ComponentError::CircularDependency("A -> B -> A".to_string()),
            ComponentError::ConfigurationError("Invalid config".to_string()),
            ComponentError::CreationError("Failed to create".to_string()),
            ComponentError::NotFound("ComponentX".to_string()),
        ];

        let expected_messages = vec![
            "Dependency 'TestDep' not found",
            "Failed to downcast dependency 'TestComponent'",
            "Circular dependency detected: A -> B -> A",
            "Configuration error: Invalid config", 
            "Component creation error: Failed to create",
            "Component not found: ComponentX",
        ];

        for (error, expected) in errors.iter().zip(expected_messages.iter()) {
            assert_eq!(error.to_string(), *expected);
        }
    }

    #[test]
    fn test_from_container_error() {
        let container_errors = vec![
            ContainerError::not_found("test component"),
            ContainerError::circular_dependency("A -> B -> A"),
            ContainerError::creation_failed("creation failed"),
            ContainerError::type_cast_failed("cast failed"),
            ContainerError::configuration("config error"),
            ContainerError::other("other error"),
        ];

        let component_errors: Vec<ComponentError> = container_errors
            .into_iter()
            .map(ComponentError::from)
            .collect();

        assert!(matches!(component_errors[0], ComponentError::NotFound(_)));
        assert!(matches!(component_errors[1], ComponentError::CircularDependency(_)));
        assert!(matches!(component_errors[2], ComponentError::CreationError(_)));
        assert!(matches!(component_errors[3], ComponentError::DowncastFailed(_)));
        assert!(matches!(component_errors[4], ComponentError::ConfigurationError(_)));
        assert!(matches!(component_errors[5], ComponentError::CreationError(_)));
    }

    #[test]
    fn test_to_container_error() {
        let component_errors = vec![
            ComponentError::NotFound("test".to_string()),
            ComponentError::CircularDependency("A -> B -> A".to_string()),
            ComponentError::CreationError("create failed".to_string()),
            ComponentError::DowncastFailed("cast failed".to_string()),
            ComponentError::ConfigurationError("config error".to_string()),
            ComponentError::DependencyNotFound("dependency".to_string()),
        ];

        let container_errors: Vec<ContainerError> = component_errors
            .into_iter()
            .map(ContainerError::from)
            .collect();

        assert_eq!(container_errors[0].kind, ContainerErrorKind::NotFound);
        assert_eq!(container_errors[1].kind, ContainerErrorKind::CircularDependency);
        assert_eq!(container_errors[2].kind, ContainerErrorKind::CreationFailed);
        assert_eq!(container_errors[3].kind, ContainerErrorKind::TypeCastFailed);
        assert_eq!(container_errors[4].kind, ContainerErrorKind::Configuration);
        assert_eq!(container_errors[5].kind, ContainerErrorKind::NotFound);
        assert!(container_errors[5].message.contains("Dependency: dependency"));
    }

    #[test]
    fn test_error_roundup_conversion() {
        let original_container_error = ContainerError::circular_dependency("A -> B -> A");
        let component_error: ComponentError = original_container_error.into();
        let back_to_container: ContainerError = component_error.into();
        
        assert_eq!(back_to_container.kind, ContainerErrorKind::CircularDependency);
        assert_eq!(back_to_container.message, "A -> B -> A");
    }
}
