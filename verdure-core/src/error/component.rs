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
