//! Container-specific error types and implementations
//! 
//! This module defines error types specifically related to IoC container operations,
//! including component resolution, dependency injection, and lifecycle management.

use std::fmt;

/// The main error type for container operations
/// 
/// `ContainerError` represents various failure conditions that can occur during
/// container initialization, component creation, and dependency resolution.
/// 
/// # Examples
/// 
/// ```rust
/// use verdure_core::error::container::{ContainerError, ContainerErrorKind};
/// 
/// let error = ContainerError::not_found("ComponentA not found");
/// println!("{}", error); // Prints: "Component not found: ComponentA not found"
/// ```
#[derive(Debug)]
pub struct ContainerError {
    /// The specific kind of error that occurred
    pub kind: ContainerErrorKind,
    /// A human-readable error message describing the issue
    pub message: String,
    /// Optional source error that caused this error
    pub source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

/// Enumeration of different types of container errors
/// 
/// This enum categorizes the various types of errors that can occur during
/// container operations, making it easier to handle different error conditions
/// appropriately.
#[derive(Debug, PartialEq, Eq)]
pub enum ContainerErrorKind {
    /// A component or dependency was not found
    NotFound,
    /// A circular dependency was detected between components
    CircularDependency,
    /// Failed to create a component instance
    CreationFailed,
    /// Failed to cast a component to the expected type
    TypeCastFailed,
    /// Configuration error occurred
    Configuration,
    /// Other unspecified error
    Other,
}

impl fmt::Display for ContainerErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContainerErrorKind::NotFound => write!(f, "Component not found"),
            ContainerErrorKind::CircularDependency => write!(f, "Circular dependency detected"),
            ContainerErrorKind::CreationFailed => write!(f, "Component creation failed"),
            ContainerErrorKind::TypeCastFailed => write!(f, "Type cast failed"),
            ContainerErrorKind::Configuration => write!(f, "Configuration error"),
            ContainerErrorKind::Other => write!(f, "Other error"),
        }
    }
}

impl fmt::Display for ContainerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.kind, self.message)
    }
}

impl std::error::Error for ContainerError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source
            .as_ref()
            .map(|e| e.as_ref() as &(dyn std::error::Error + 'static))
    }
}

impl ContainerError {
    /// Creates a new ContainerError with the specified kind and message
    /// 
    /// # Arguments
    /// 
    /// * `kind` - The kind of error that occurred
    /// * `message` - A descriptive error message
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use verdure_core::error::container::{ContainerError, ContainerErrorKind};
    /// 
    /// let error = ContainerError::new(ContainerErrorKind::NotFound, "Component missing");
    /// ```
    pub fn new<M: Into<String>>(kind: ContainerErrorKind, message: M) -> Self {
        Self {
            kind,
            message: message.into(),
            source: None,
        }
    }

    /// Adds a source error to this ContainerError
    /// 
    /// This is useful for error chaining, where one error is caused by another.
    /// 
    /// # Arguments
    /// 
    /// * `err` - The source error that caused this error
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use verdure_core::error::container::ContainerError;
    /// use std::io::Error;
    /// 
    /// let io_error = Error::new(std::io::ErrorKind::NotFound, "file not found");
    /// let container_error = ContainerError::creation_failed("Failed to load config")
    ///     .with_source(io_error);
    /// ```
    pub fn with_source<E>(mut self, err: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        self.source = Some(Box::new(err));
        self
    }

    /// Creates a new "not found" error
    /// 
    /// # Arguments
    /// 
    /// * `message` - A descriptive message about what was not found
    pub fn not_found<M: Into<String>>(message: M) -> Self {
        Self::new(ContainerErrorKind::NotFound, message)
    }

    /// Creates a new "circular dependency" error
    /// 
    /// # Arguments
    /// 
    /// * `message` - A descriptive message about the circular dependency
    pub fn circular_dependency<M: Into<String>>(message: M) -> Self {
        Self::new(ContainerErrorKind::CircularDependency, message)
    }

    /// Creates a new "creation failed" error
    /// 
    /// # Arguments
    /// 
    /// * `message` - A descriptive message about the creation failure
    pub fn creation_failed<M: Into<String>>(message: M) -> Self {
        Self::new(ContainerErrorKind::CreationFailed, message)
    }

    /// Creates a new "type cast failed" error
    /// 
    /// # Arguments
    /// 
    /// * `message` - A descriptive message about the type cast failure
    pub fn type_cast_failed<M: Into<String>>(message: M) -> Self {
        Self::new(ContainerErrorKind::TypeCastFailed, message)
    }

    /// Creates a new "configuration" error
    /// 
    /// # Arguments
    /// 
    /// * `message` - A descriptive message about the configuration error
    pub fn configuration<M: Into<String>>(message: M) -> Self {
        Self::new(ContainerErrorKind::Configuration, message)
    }

    /// Creates a new "other" error
    /// 
    /// # Arguments
    /// 
    /// * `message` - A descriptive message about the error
    pub fn other<M: Into<String>>(message: M) -> Self {
        Self::new(ContainerErrorKind::Other, message)
    }
}

impl From<&str> for ContainerError {
    fn from(msg: &str) -> Self {
        ContainerError::new(ContainerErrorKind::Other, msg)
    }
}

impl From<String> for ContainerError {
    fn from(msg: String) -> Self {
        ContainerError::new(ContainerErrorKind::Other, msg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_container_error_creation() {
        let error = ContainerError::not_found("test component");
        assert_eq!(error.kind, ContainerErrorKind::NotFound);
        assert_eq!(error.message, "test component");
        assert!(error.source.is_none());
    }

    #[test]
    fn test_container_error_display() {
        let error = ContainerError::circular_dependency("ComponentA -> ComponentB -> ComponentA");
        assert_eq!(
            error.to_string(),
            "Circular dependency detected: ComponentA -> ComponentB -> ComponentA"
        );
    }

    #[test]
    fn test_container_error_with_source() {
        let source_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let error = ContainerError::creation_failed("Failed to create component")
            .with_source(source_error);
        
        assert_eq!(error.kind, ContainerErrorKind::CreationFailed);
        assert!(error.source.is_some());
    }

    #[test]
    fn test_container_error_from_string() {
        let error: ContainerError = "test error message".into();
        assert_eq!(error.kind, ContainerErrorKind::Other);
        assert_eq!(error.message, "test error message");
    }

    #[test]
    fn test_container_error_from_owned_string() {
        let msg = String::from("test error message");
        let error: ContainerError = msg.into();
        assert_eq!(error.kind, ContainerErrorKind::Other);
        assert_eq!(error.message, "test error message");
    }

    #[test]
    fn test_all_error_kinds() {
        let errors = vec![
            ContainerError::not_found("not found"),
            ContainerError::circular_dependency("circular"),
            ContainerError::creation_failed("creation failed"),
            ContainerError::type_cast_failed("cast failed"),
            ContainerError::configuration("config error"),
            ContainerError::other("other error"),
        ];

        let expected_kinds = vec![
            ContainerErrorKind::NotFound,
            ContainerErrorKind::CircularDependency,
            ContainerErrorKind::CreationFailed,
            ContainerErrorKind::TypeCastFailed,
            ContainerErrorKind::Configuration,
            ContainerErrorKind::Other,
        ];

        for (error, expected_kind) in errors.iter().zip(expected_kinds.iter()) {
            assert_eq!(&error.kind, expected_kind);
        }
    }

    #[test]
    fn test_error_kind_display() {
        assert_eq!(ContainerErrorKind::NotFound.to_string(), "Component not found");
        assert_eq!(ContainerErrorKind::CircularDependency.to_string(), "Circular dependency detected");
        assert_eq!(ContainerErrorKind::CreationFailed.to_string(), "Component creation failed");
        assert_eq!(ContainerErrorKind::TypeCastFailed.to_string(), "Type cast failed");
        assert_eq!(ContainerErrorKind::Configuration.to_string(), "Configuration error");
        assert_eq!(ContainerErrorKind::Other.to_string(), "Other error");
    }
}
