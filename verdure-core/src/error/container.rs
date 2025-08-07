use std::fmt;
#[derive(Debug)]
pub struct ContainerError {
    pub kind: ContainerErrorKind,
    pub message: String,
    pub source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ContainerErrorKind {
    NotFound,
    CircularDependency,
    CreationFailed,
    TypeCastFailed,
    Configuration,
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
    pub fn new<M: Into<String>>(kind: ContainerErrorKind, message: M) -> Self {
        Self {
            kind,
            message: message.into(),
            source: None,
        }
    }

    pub fn with_source<E>(mut self, err: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        self.source = Some(Box::new(err));
        self
    }

    pub fn not_found<M: Into<String>>(message: M) -> Self {
        Self::new(ContainerErrorKind::NotFound, message)
    }

    pub fn circular_dependency<M: Into<String>>(message: M) -> Self {
        Self::new(ContainerErrorKind::CircularDependency, message)
    }

    pub fn creation_failed<M: Into<String>>(message: M) -> Self {
        Self::new(ContainerErrorKind::CreationFailed, message)
    }

    pub fn type_cast_failed<M: Into<String>>(message: M) -> Self {
        Self::new(ContainerErrorKind::TypeCastFailed, message)
    }

    pub fn configuration<M: Into<String>>(message: M) -> Self {
        Self::new(ContainerErrorKind::Configuration, message)
    }

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
