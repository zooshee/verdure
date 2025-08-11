//! Context error types
//!
//! This module defines error types used throughout the Verdure context system,
//! providing comprehensive error handling for context operations, configuration
//! management, and environment handling.

use std::fmt;

/// Context operation errors
///
/// `ContextError` represents various error conditions that can occur during
/// context operations, configuration management, and environment handling.
///
/// # Examples
///
/// ```rust
/// use verdure_context::ContextError;
///
/// let error = ContextError::configuration_not_found("database.url");
/// assert!(matches!(error, ContextError::ConfigurationNotFound { .. }));
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum ContextError {
    /// Configuration key not found
    ConfigurationNotFound {
        /// The configuration key that was not found
        key: String,
    },

    /// Invalid configuration format or value
    InvalidConfiguration {
        /// The configuration key with invalid value
        key: String,
        /// Reason for the invalid configuration
        reason: String,
    },

    /// Environment profile not found
    ProfileNotFound {
        /// The profile name that was not found
        profile: String,
    },

    /// Context initialization failed
    InitializationFailed {
        /// Reason for initialization failure
        reason: String,
    },

    /// Configuration file I/O error
    ConfigurationFileError {
        /// Error message
        message: String,
    },

    /// Serialization/deserialization error
    SerializationError {
        /// Error message
        message: String,
    },

    /// Property binding error
    PropertyBindingError {
        /// Property name
        property: String,
        /// Binding error reason
        reason: String,
    },
}

impl fmt::Display for ContextError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContextError::ConfigurationNotFound { key } => {
                write!(f, "Configuration not found: {}", key)
            }
            ContextError::InvalidConfiguration { key, reason } => {
                write!(f, "Invalid configuration for key '{}': {}", key, reason)
            }
            ContextError::ProfileNotFound { profile } => {
                write!(f, "Environment profile not found: {}", profile)
            }
            ContextError::InitializationFailed { reason } => {
                write!(f, "Context initialization failed: {}", reason)
            }
            ContextError::ConfigurationFileError { message } => {
                write!(f, "Configuration file error: {}", message)
            }
            ContextError::SerializationError { message } => {
                write!(f, "Serialization error: {}", message)
            }
            ContextError::PropertyBindingError { property, reason } => {
                write!(f, "Property binding error for '{}': {}", property, reason)
            }
        }
    }
}

impl std::error::Error for ContextError {}

impl ContextError {
    /// Creates a configuration not found error
    ///
    /// # Arguments
    ///
    /// * `key` - The configuration key that was not found
    ///
    /// # Examples
    ///
    /// ```rust
    /// use verdure_context::ContextError;
    ///
    /// let error = ContextError::configuration_not_found("app.name");
    /// ```
    pub fn configuration_not_found(key: impl Into<String>) -> Self {
        Self::ConfigurationNotFound { key: key.into() }
    }

    /// Creates an invalid configuration error
    ///
    /// # Arguments
    ///
    /// * `key` - The configuration key with invalid value
    /// * `reason` - Reason for the invalid configuration
    pub fn invalid_configuration(key: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::InvalidConfiguration {
            key: key.into(),
            reason: reason.into(),
        }
    }

    /// Creates a profile not found error
    ///
    /// # Arguments
    ///
    /// * `profile` - The profile name that was not found
    pub fn profile_not_found(profile: impl Into<String>) -> Self {
        Self::ProfileNotFound {
            profile: profile.into(),
        }
    }

    /// Creates an initialization failed error
    ///
    /// # Arguments
    ///
    /// * `reason` - Reason for initialization failure
    pub fn initialization_failed(reason: impl Into<String>) -> Self {
        Self::InitializationFailed {
            reason: reason.into(),
        }
    }

    /// Creates a configuration file error
    ///
    /// # Arguments
    ///
    /// * `message` - Error message
    pub fn configuration_file_error(message: impl Into<String>) -> Self {
        Self::ConfigurationFileError {
            message: message.into(),
        }
    }

    /// Creates a serialization error
    ///
    /// # Arguments
    ///
    /// * `message` - Error message
    pub fn serialization_error(message: impl Into<String>) -> Self {
        Self::SerializationError {
            message: message.into(),
        }
    }

    /// Creates a property binding error
    ///
    /// # Arguments
    ///
    /// * `property` - Property name
    /// * `reason` - Binding error reason
    pub fn property_binding_error(property: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::PropertyBindingError {
            property: property.into(),
            reason: reason.into(),
        }
    }
}

/// Result type for context operations
///
/// A convenience type alias for `Result<T, ContextError>` used throughout
/// the context system.
pub type ContextResult<T> = Result<T, ContextError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_error_creation() {
        let error = ContextError::configuration_not_found("test.key");
        assert!(matches!(error, ContextError::ConfigurationNotFound { .. }));
        assert_eq!(error.to_string(), "Configuration not found: test.key");
    }

    #[test]
    fn test_invalid_configuration_error() {
        let error = ContextError::invalid_configuration("port", "not a number");
        assert!(matches!(error, ContextError::InvalidConfiguration { .. }));
        assert_eq!(
            error.to_string(),
            "Invalid configuration for key 'port': not a number"
        );
    }

    #[test]
    fn test_profile_not_found_error() {
        let error = ContextError::profile_not_found("development");
        assert!(matches!(error, ContextError::ProfileNotFound { .. }));
        assert_eq!(
            error.to_string(),
            "Environment profile not found: development"
        );
    }

    #[test]
    fn test_initialization_failed_error() {
        let error = ContextError::initialization_failed("missing required config");
        assert!(matches!(error, ContextError::InitializationFailed { .. }));
        assert_eq!(
            error.to_string(),
            "Context initialization failed: missing required config"
        );
    }

    #[test]
    fn test_configuration_file_error() {
        let error = ContextError::configuration_file_error("file not found");
        assert!(matches!(error, ContextError::ConfigurationFileError { .. }));
        assert_eq!(
            error.to_string(),
            "Configuration file error: file not found"
        );
    }

    #[test]
    fn test_serialization_error() {
        let error = ContextError::serialization_error("invalid TOML format");
        assert!(matches!(error, ContextError::SerializationError { .. }));
        assert_eq!(
            error.to_string(),
            "Serialization error: invalid TOML format"
        );
    }

    #[test]
    fn test_property_binding_error() {
        let error = ContextError::property_binding_error("database.port", "type mismatch");
        assert!(matches!(error, ContextError::PropertyBindingError { .. }));
        assert_eq!(
            error.to_string(),
            "Property binding error for 'database.port': type mismatch"
        );
    }
}
