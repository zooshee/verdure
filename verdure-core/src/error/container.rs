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
