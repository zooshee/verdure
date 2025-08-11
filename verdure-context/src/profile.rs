//! Environment profiles management
//!
//! This module provides environment profile functionality for the Verdure context system.
//! Environment profiles allow different configurations for different deployment environments
//! (development, testing, staging, production, etc.).

use crate::error::{ContextError, ContextResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Environment profile definition
///
/// `Profile` represents a specific environment configuration profile with its own
/// set of configuration properties. Profiles enable environment-specific behavior
/// and configuration management.
///
/// # Examples
///
/// ```rust
/// use verdure_context::Profile;
/// use std::collections::HashMap;
///
/// let mut properties = HashMap::new();
/// properties.insert("database.url".to_string(), "postgres://localhost:5432/dev".to_string());
///
/// let profile = Profile::new("development", properties);
/// assert_eq!(profile.name(), "development");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    /// Profile name (e.g., "development", "production")
    name: String,
    /// Profile-specific configuration properties
    properties: HashMap<String, String>,
    /// Whether this profile is active
    active: bool,
}

impl Profile {
    /// Creates a new profile with the given name and properties
    ///
    /// # Arguments
    ///
    /// * `name` - The profile name
    /// * `properties` - Configuration properties for this profile
    ///
    /// # Examples
    ///
    /// ```rust
    /// use verdure_context::Profile;
    /// use std::collections::HashMap;
    ///
    /// let mut props = HashMap::new();
    /// props.insert("app.name".to_string(), "MyApp".to_string());
    ///
    /// let profile = Profile::new("production", props);
    /// ```
    pub fn new(name: impl Into<String>, properties: HashMap<String, String>) -> Self {
        Self {
            name: name.into(),
            properties,
            active: false,
        }
    }

    /// Returns the profile name
    ///
    /// # Examples
    ///
    /// ```rust
    /// use verdure_context::Profile;
    /// use std::collections::HashMap;
    ///
    /// let profile = Profile::new("test", HashMap::new());
    /// assert_eq!(profile.name(), "test");
    /// ```
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns whether this profile is active
    ///
    /// # Examples
    ///
    /// ```rust
    /// use verdure_context::Profile;
    /// use std::collections::HashMap;
    ///
    /// let profile = Profile::new("test", HashMap::new());
    /// assert!(!profile.is_active());
    /// ```
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Sets the active status of this profile
    ///
    /// # Arguments
    ///
    /// * `active` - Whether to make this profile active
    pub fn set_active(&mut self, active: bool) {
        self.active = active;
    }

    /// Gets a property value by key
    ///
    /// # Arguments
    ///
    /// * `key` - The property key
    ///
    /// # Returns
    ///
    /// The property value if found, `None` otherwise
    ///
    /// # Examples
    ///
    /// ```rust
    /// use verdure_context::Profile;
    /// use std::collections::HashMap;
    ///
    /// let mut props = HashMap::new();
    /// props.insert("app.port".to_string(), "8080".to_string());
    ///
    /// let profile = Profile::new("dev", props);
    /// assert_eq!(profile.get_property("app.port"), Some("8080"));
    /// assert_eq!(profile.get_property("missing"), None);
    /// ```
    pub fn get_property(&self, key: &str) -> Option<&str> {
        self.properties.get(key).map(|s| s.as_str())
    }

    /// Sets a property value
    ///
    /// # Arguments
    ///
    /// * `key` - The property key
    /// * `value` - The property value
    pub fn set_property(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.properties.insert(key.into(), value.into());
    }

    /// Returns all properties in this profile
    ///
    /// # Examples
    ///
    /// ```rust
    /// use verdure_context::Profile;
    /// use std::collections::HashMap;
    ///
    /// let mut props = HashMap::new();
    /// props.insert("key1".to_string(), "value1".to_string());
    ///
    /// let profile = Profile::new("test", props);
    /// let properties = profile.properties();
    /// assert_eq!(properties.get("key1"), Some(&"value1".to_string()));
    /// ```
    pub fn properties(&self) -> &HashMap<String, String> {
        &self.properties
    }

    /// Returns a mutable reference to all properties
    pub fn properties_mut(&mut self) -> &mut HashMap<String, String> {
        &mut self.properties
    }
}

/// Profile manager for handling multiple environment profiles
///
/// `ProfileManager` manages a collection of environment profiles and provides
/// functionality to activate profiles, resolve property values with profile
/// precedence, and manage profile-specific configurations.
///
/// # Examples
///
/// ```rust
/// use verdure_context::{ProfileManager, Profile};
/// use std::collections::HashMap;
///
/// let mut manager = ProfileManager::new();
///
/// let mut dev_props = HashMap::new();
/// dev_props.insert("database.url".to_string(), "postgres://localhost/dev".to_string());
///
/// let dev_profile = Profile::new("development", dev_props);
/// manager.add_profile(dev_profile).unwrap();
/// manager.activate_profile("development").unwrap();
/// ```
#[derive(Debug, Clone)]
pub struct ProfileManager {
    /// All available profiles
    profiles: HashMap<String, Profile>,
    /// Currently active profile names
    active_profiles: Vec<String>,
}

impl ProfileManager {
    /// Creates a new profile manager
    ///
    /// # Examples
    ///
    /// ```rust
    /// use verdure_context::ProfileManager;
    ///
    /// let manager = ProfileManager::new();
    /// ```
    pub fn new() -> Self {
        Self {
            profiles: HashMap::new(),
            active_profiles: Vec::new(),
        }
    }

    /// Adds a profile to the manager
    ///
    /// # Arguments
    ///
    /// * `profile` - The profile to add
    ///
    /// # Errors
    ///
    /// Returns an error if a profile with the same name already exists
    ///
    /// # Examples
    ///
    /// ```rust
    /// use verdure_context::{ProfileManager, Profile};
    /// use std::collections::HashMap;
    ///
    /// let mut manager = ProfileManager::new();
    /// let profile = Profile::new("test", HashMap::new());
    ///
    /// manager.add_profile(profile).unwrap();
    /// ```
    pub fn add_profile(&mut self, profile: Profile) -> ContextResult<()> {
        let name = profile.name().to_string();
        if self.profiles.contains_key(&name) {
            return Err(ContextError::invalid_configuration(
                format!("profile.{}", name),
                "Profile already exists",
            ));
        }
        self.profiles.insert(name, profile);
        Ok(())
    }

    /// Activates a profile by name
    ///
    /// # Arguments
    ///
    /// * `profile_name` - Name of the profile to activate
    ///
    /// # Errors
    ///
    /// Returns an error if the profile doesn't exist
    ///
    /// # Examples
    ///
    /// ```rust
    /// use verdure_context::{ProfileManager, Profile};
    /// use std::collections::HashMap;
    ///
    /// let mut manager = ProfileManager::new();
    /// let profile = Profile::new("production", HashMap::new());
    /// manager.add_profile(profile).unwrap();
    ///
    /// manager.activate_profile("production").unwrap();
    /// ```
    pub fn activate_profile(&mut self, profile_name: &str) -> ContextResult<()> {
        if !self.profiles.contains_key(profile_name) {
            return Err(ContextError::profile_not_found(profile_name));
        }

        // Mark profile as active
        if let Some(profile) = self.profiles.get_mut(profile_name) {
            profile.set_active(true);
        }

        // Add to active profiles if not already present
        if !self.active_profiles.contains(&profile_name.to_string()) {
            self.active_profiles.push(profile_name.to_string());
        }

        Ok(())
    }

    /// Deactivates a profile by name
    ///
    /// # Arguments
    ///
    /// * `profile_name` - Name of the profile to deactivate
    pub fn deactivate_profile(&mut self, profile_name: &str) {
        // Mark profile as inactive
        if let Some(profile) = self.profiles.get_mut(profile_name) {
            profile.set_active(false);
        }

        // Remove from active profiles
        self.active_profiles.retain(|name| name != profile_name);
    }

    /// Gets the number of properties in a specific profile
    ///
    /// # Arguments
    ///
    /// * `profile_name` - Name of the profile
    ///
    /// # Returns
    ///
    /// The number of properties in the profile, or 0 if profile doesn't exist
    pub fn get_profile_properties_count(&self, profile_name: &str) -> usize {
        self.profiles
            .get(profile_name)
            .map(|profile| profile.properties().len())
            .unwrap_or(0)
    }

    /// Gets a property value, checking active profiles in order
    ///
    /// Properties in profiles activated later take precedence over earlier ones.
    ///
    /// # Arguments
    ///
    /// * `key` - The property key to look up
    ///
    /// # Returns
    ///
    /// The property value if found in any active profile, `None` otherwise
    ///
    /// # Examples
    ///
    /// ```rust
    /// use verdure_context::{ProfileManager, Profile};
    /// use std::collections::HashMap;
    ///
    /// let mut manager = ProfileManager::new();
    ///
    /// let mut dev_props = HashMap::new();
    /// dev_props.insert("app.port".to_string(), "3000".to_string());
    /// let dev_profile = Profile::new("dev", dev_props);
    ///
    /// manager.add_profile(dev_profile).unwrap();
    /// manager.activate_profile("dev").unwrap();
    ///
    /// assert_eq!(manager.get_property("app.port"), Some("3000"));
    /// ```
    pub fn get_property(&self, key: &str) -> Option<&str> {
        // Check active profiles in reverse order (last activated has highest precedence)
        for profile_name in self.active_profiles.iter().rev() {
            if let Some(profile) = self.profiles.get(profile_name) {
                if let Some(value) = profile.get_property(key) {
                    return Some(value);
                }
            }
        }
        None
    }

    /// Returns the names of all active profiles
    ///
    /// # Examples
    ///
    /// ```rust
    /// use verdure_context::{ProfileManager, Profile};
    /// use std::collections::HashMap;
    ///
    /// let mut manager = ProfileManager::new();
    /// let profile = Profile::new("staging", HashMap::new());
    /// manager.add_profile(profile).unwrap();
    /// manager.activate_profile("staging").unwrap();
    ///
    /// assert_eq!(manager.active_profiles(), &["staging"]);
    /// ```
    pub fn active_profiles(&self) -> &[String] {
        &self.active_profiles
    }

    /// Returns all available profiles
    ///
    /// # Examples
    ///
    /// ```rust
    /// use verdure_context::{ProfileManager, Profile};
    /// use std::collections::HashMap;
    ///
    /// let mut manager = ProfileManager::new();
    /// let profile = Profile::new("test", HashMap::new());
    /// manager.add_profile(profile).unwrap();
    ///
    /// assert!(manager.profiles().contains_key("test"));
    /// ```
    pub fn profiles(&self) -> &HashMap<String, Profile> {
        &self.profiles
    }

    /// Gets a profile by name
    ///
    /// # Arguments
    ///
    /// * `name` - The profile name
    ///
    /// # Returns
    ///
    /// A reference to the profile if found, `None` otherwise
    pub fn get_profile(&self, name: &str) -> Option<&Profile> {
        self.profiles.get(name)
    }

    /// Gets a mutable reference to a profile by name
    ///
    /// # Arguments
    ///
    /// * `name` - The profile name
    ///
    /// # Returns
    ///
    /// A mutable reference to the profile if found, `None` otherwise
    pub fn get_profile_mut(&mut self, name: &str) -> Option<&mut Profile> {
        self.profiles.get_mut(name)
    }
}

impl Default for ProfileManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_creation() {
        let mut props = HashMap::new();
        props.insert("key1".to_string(), "value1".to_string());

        let profile = Profile::new("test", props);
        assert_eq!(profile.name(), "test");
        assert!(!profile.is_active());
        assert_eq!(profile.get_property("key1"), Some("value1"));
        assert_eq!(profile.get_property("missing"), None);
    }

    #[test]
    fn test_profile_active_status() {
        let mut profile = Profile::new("test", HashMap::new());
        assert!(!profile.is_active());

        profile.set_active(true);
        assert!(profile.is_active());

        profile.set_active(false);
        assert!(!profile.is_active());
    }

    #[test]
    fn test_profile_property_management() {
        let mut profile = Profile::new("test", HashMap::new());

        profile.set_property("new_key", "new_value");
        assert_eq!(profile.get_property("new_key"), Some("new_value"));

        profile.set_property("new_key", "updated_value");
        assert_eq!(profile.get_property("new_key"), Some("updated_value"));
    }

    #[test]
    fn test_profile_manager_creation() {
        let manager = ProfileManager::new();
        assert!(manager.active_profiles().is_empty());
        assert!(manager.profiles().is_empty());
    }

    #[test]
    fn test_profile_manager_add_profile() {
        let mut manager = ProfileManager::new();
        let profile = Profile::new("dev", HashMap::new());

        manager.add_profile(profile).unwrap();
        assert!(manager.profiles().contains_key("dev"));
    }

    #[test]
    fn test_profile_manager_duplicate_profile() {
        let mut manager = ProfileManager::new();
        let profile1 = Profile::new("dev", HashMap::new());
        let profile2 = Profile::new("dev", HashMap::new());

        manager.add_profile(profile1).unwrap();
        let result = manager.add_profile(profile2);
        assert!(result.is_err());
    }

    #[test]
    fn test_profile_activation() {
        let mut manager = ProfileManager::new();
        let profile = Profile::new("prod", HashMap::new());

        manager.add_profile(profile).unwrap();
        manager.activate_profile("prod").unwrap();

        assert_eq!(manager.active_profiles(), &["prod"]);
        assert!(manager.get_profile("prod").unwrap().is_active());
    }

    #[test]
    fn test_profile_activation_nonexistent() {
        let mut manager = ProfileManager::new();
        let result = manager.activate_profile("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_profile_deactivation() {
        let mut manager = ProfileManager::new();
        let profile = Profile::new("test", HashMap::new());

        manager.add_profile(profile).unwrap();
        manager.activate_profile("test").unwrap();
        assert!(manager.get_profile("test").unwrap().is_active());

        manager.deactivate_profile("test");
        assert!(!manager.get_profile("test").unwrap().is_active());
        assert!(manager.active_profiles().is_empty());
    }

    #[test]
    fn test_property_resolution_precedence() {
        let mut manager = ProfileManager::new();

        // Create first profile
        let mut props1 = HashMap::new();
        props1.insert("app.name".to_string(), "App1".to_string());
        props1.insert("app.port".to_string(), "8080".to_string());
        let profile1 = Profile::new("profile1", props1);

        // Create second profile with overlapping property
        let mut props2 = HashMap::new();
        props2.insert("app.name".to_string(), "App2".to_string());
        props2.insert("app.version".to_string(), "2.0".to_string());
        let profile2 = Profile::new("profile2", props2);

        manager.add_profile(profile1).unwrap();
        manager.add_profile(profile2).unwrap();

        manager.activate_profile("profile1").unwrap();
        manager.activate_profile("profile2").unwrap();

        // profile2 was activated last, so it should take precedence
        assert_eq!(manager.get_property("app.name"), Some("App2"));
        assert_eq!(manager.get_property("app.port"), Some("8080")); // Only in profile1
        assert_eq!(manager.get_property("app.version"), Some("2.0")); // Only in profile2
        assert_eq!(manager.get_property("missing"), None);
    }

    #[test]
    fn test_multiple_active_profiles() {
        let mut manager = ProfileManager::new();

        let profile1 = Profile::new("dev", HashMap::new());
        let profile2 = Profile::new("local", HashMap::new());

        manager.add_profile(profile1).unwrap();
        manager.add_profile(profile2).unwrap();

        manager.activate_profile("dev").unwrap();
        manager.activate_profile("local").unwrap();

        assert_eq!(manager.active_profiles(), &["dev", "local"]);
    }
}
