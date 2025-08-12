//! Event broadcasting system
//!
//! This module provides a comprehensive event broadcasting system for the Verdure context.
//! It supports application-wide event publishing and subscription, enabling decoupled
//! communication between different parts of the application.

use dashmap::DashMap;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;

/// Built-in context lifecycle events

/// Event fired when the application context starts initialization
///
/// This event is published at the very beginning of the context initialization
/// process, before any actual initialization work begins.
#[derive(Debug, Clone)]
pub struct ContextInitializingEvent {
    /// Number of configuration sources to be loaded
    pub config_sources_count: usize,
    /// Number of profiles to be activated
    pub active_profiles_count: usize,
    /// Initialization start timestamp
    pub timestamp: std::time::SystemTime,
}

impl Event for ContextInitializingEvent {
    fn name(&self) -> &'static str {
        "ContextInitializing"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn into_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }
}

/// Event fired when the application context is initialized
///
/// This event is published after the context has been fully initialized,
/// including all configuration sources, profiles, and the IoC container.
#[derive(Debug, Clone)]
pub struct ContextInitializedEvent {
    /// Number of configuration sources loaded
    pub config_sources_count: usize,
    /// Number of active profiles
    pub active_profiles_count: usize,
    /// Initialization timestamp
    pub timestamp: std::time::SystemTime,
}

impl Event for ContextInitializedEvent {
    fn name(&self) -> &'static str {
        "ContextInitialized"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn into_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }
}

/// Event fired when a profile is activated
///
/// This event is published whenever a new profile is activated in the context.
#[derive(Debug, Clone)]
pub struct ProfileActivatedEvent {
    /// Name of the activated profile
    pub profile_name: String,
    /// Properties count in the activated profile
    pub properties_count: usize,
    /// Activation timestamp
    pub timestamp: std::time::SystemTime,
}

impl Event for ProfileActivatedEvent {
    fn name(&self) -> &'static str {
        "ProfileActivated"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn into_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }
}

/// Event fired when configuration is changed at runtime
///
/// This event is published when configuration values are updated after
/// the initial context setup.
#[derive(Debug, Clone)]
pub struct ConfigurationChangedEvent {
    /// The configuration key that was changed
    pub key: String,
    /// The old value (if any)
    pub old_value: Option<String>,
    /// The new value
    pub new_value: String,
    /// Change timestamp
    pub timestamp: std::time::SystemTime,
}

impl Event for ConfigurationChangedEvent {
    fn name(&self) -> &'static str {
        "ConfigurationChanged"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn into_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }
}

/// Event trait that all events must implement
///
/// This trait allows events to be stored and transmitted in a type-safe manner
/// while supporting downcasting to the concrete event type.
///
/// # Examples
///
/// ```rust
/// use verdure_context::Event;
/// use std::any::Any;
///
/// #[derive(Debug, Clone)]
/// struct UserCreatedEvent {
///     pub user_id: u64,
///     pub username: String,
/// }
///
/// impl Event for UserCreatedEvent {
///     fn name(&self) -> &'static str {
///         "UserCreated"
///     }
///     
///     fn as_any(&self) -> &dyn Any {
///         self
///     }
///     
///     fn into_any(self: Box<Self>) -> Box<dyn Any> {
///         self
///     }
/// }
/// ```
pub trait Event: Any + Send + Sync {
    /// Returns the name of the event type
    ///
    /// This should be a unique identifier for the event type.
    fn name(&self) -> &'static str;

    /// Returns the event as `Any` for downcasting
    fn as_any(&self) -> &dyn Any;

    /// Converts the event to `Any` for downcasting (consuming)
    fn into_any(self: Box<Self>) -> Box<dyn Any>;
}

/// Event listener trait
///
/// Implement this trait to handle specific event types. The listener will be
/// called whenever an event of the specified type is published.
///
/// # Examples
///
/// ```rust
/// use verdure_context::{Event, EventListener};
/// use std::any::Any;
///
/// #[derive(Debug, Clone)]
/// struct MyEvent {
///     data: String,
/// }
///
/// impl Event for MyEvent {
///     fn name(&self) -> &'static str {
///         "MyEvent"
///     }
///     
///     fn as_any(&self) -> &dyn Any {
///         self
///     }
///     
///     fn into_any(self: Box<Self>) -> Box<dyn Any> {
///         self
///     }
/// }
///
/// struct MyEventListener;
///
/// impl EventListener<MyEvent> for MyEventListener {
///     fn on_event(&self, event: &MyEvent) {
///         println!("Received event with data: {}", event.data);
///     }
/// }
/// ```
pub trait EventListener<T: Event>: Send + Sync {
    /// Called when an event of type `T` is published
    ///
    /// # Arguments
    ///
    /// * `event` - The event instance
    fn on_event(&self, event: &T);
}

/// Type-erased event listener
///
/// This allows storing listeners of different event types in the same collection.
pub trait AnyEventListener: Send + Sync {
    /// Handles an event if the listener is registered for this event type
    ///
    /// # Arguments
    ///
    /// * `event` - The event to handle
    ///
    /// # Returns
    ///
    /// `true` if the listener handled the event, `false` otherwise
    fn handle_event(&self, event: &dyn Event) -> bool;

    /// Returns the TypeId of the event type this listener handles
    fn event_type_id(&self) -> TypeId;
}

/// Context-aware event listener trait for lifecycle events
///
/// This trait allows event listeners to receive a reference to the ApplicationContext
/// along with the event, enabling them to interact with the context during lifecycle events.
///
/// # Examples
///
/// ```rust
/// use verdure_context::{ContextAwareEventListener, ContextInitializedEvent, ApplicationContext};
///
/// struct StartupListener;
///
/// impl ContextAwareEventListener<ContextInitializedEvent> for StartupListener {
///     fn on_context_event(&self, event: &ContextInitializedEvent, context: &ApplicationContext) {
///         println!("Context initialized with {} sources", event.config_sources_count);
///         println!("App name: {}", context.get_config("app.name"));
///         
///         // Can access IoC container
///         let container = context.container();
///         // Can access configuration
///         let environment = context.environment();
///         
///         println!("Running in environment: {}", environment);
///     }
/// }
/// ```
pub trait ContextAwareEventListener<T: Event>: Send + Sync {
    /// Called when an event of type `T` is published, with access to the ApplicationContext
    ///
    /// # Arguments
    ///
    /// * `event` - The event instance
    /// * `context` - Reference to the ApplicationContext
    fn on_context_event(&self, event: &T, context: &crate::context::ApplicationContext);
}

/// Type-erased context-aware event listener
///
/// This allows storing context-aware listeners of different event types in the same collection.
///
/// This allows storing listeners of different event types in the same collection.
/// Type-erased context-aware event listener
///
/// This allows storing context-aware listeners of different event types in the same collection.
pub trait AnyContextAwareEventListener: Send + Sync {
    /// Handles an event with context access if the listener is registered for this event type
    ///
    /// # Arguments
    ///
    /// * `event` - The event to handle
    /// * `context` - Reference to the ApplicationContext
    ///
    /// # Returns
    ///
    /// `true` if the listener handled the event, `false` otherwise
    fn handle_context_event(
        &self,
        event: &dyn Event,
        context: &crate::context::ApplicationContext,
    ) -> bool;

    /// Returns the TypeId of the event type this listener handles
    fn event_type_id(&self) -> TypeId;
}

/// Implementation of `AnyContextAwareEventListener` for specific context-aware event listeners
struct TypedContextAwareEventListener<T: Event, L: ContextAwareEventListener<T>> {
    listener: L,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: Event, L: ContextAwareEventListener<T>> TypedContextAwareEventListener<T, L> {
    fn new(listener: L) -> Self {
        Self {
            listener,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T: Event + 'static, L: ContextAwareEventListener<T>> AnyContextAwareEventListener
    for TypedContextAwareEventListener<T, L>
{
    fn handle_context_event(
        &self,
        event: &dyn Event,
        context: &crate::context::ApplicationContext,
    ) -> bool {
        if let Some(typed_event) = event.as_any().downcast_ref::<T>() {
            self.listener.on_context_event(typed_event, context);
            true
        } else {
            false
        }
    }

    fn event_type_id(&self) -> TypeId {
        TypeId::of::<T>()
    }
}

/// Implementation of `AnyEventListener` for specific event listeners
struct TypedEventListener<T: Event, L: EventListener<T>> {
    listener: L,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: Event + 'static, L: EventListener<T>> TypedEventListener<T, L> {
    fn new(listener: L) -> Self {
        Self {
            listener,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T: Event + 'static, L: EventListener<T>> AnyEventListener for TypedEventListener<T, L> {
    fn handle_event(&self, event: &dyn Event) -> bool {
        if let Some(typed_event) = event.as_any().downcast_ref::<T>() {
            self.listener.on_event(typed_event);
            true
        } else {
            false
        }
    }

    fn event_type_id(&self) -> TypeId {
        TypeId::of::<T>()
    }
}

/// Event publisher for broadcasting events
///
/// `EventPublisher` manages event listeners and provides functionality to publish
/// events to all registered listeners. It supports type-safe event handling and
/// allows multiple listeners for the same event type.
///
/// # Examples
///
/// ```rust
/// use verdure_context::{EventPublisher, Event, EventListener};
/// use std::any::Any;
///
/// #[derive(Debug, Clone)]
/// struct TestEvent {
///     message: String,
/// }
///
/// impl Event for TestEvent {
///     fn name(&self) -> &'static str {
///         "TestEvent"
///     }
///     
///     fn as_any(&self) -> &dyn Any {
///         self
///     }
///     
///     fn into_any(self: Box<Self>) -> Box<dyn Any> {
///         self
///     }
/// }
///
/// struct TestListener;
///
/// impl EventListener<TestEvent> for TestListener {
///     fn on_event(&self, event: &TestEvent) {
///         println!("Received: {}", event.message);
///     }
/// }
///
/// let mut publisher = EventPublisher::new();
/// publisher.subscribe(TestListener);
///
/// let event = TestEvent {
///     message: "Hello, World!".to_string(),
/// };
/// publisher.publish(&event);
/// ```
pub struct EventPublisher {
    /// Event listeners organized by event type
    listeners: DashMap<TypeId, Vec<Arc<dyn AnyEventListener>>>,
    /// Context-aware event listeners organized by event type
    context_aware_listeners: DashMap<TypeId, Vec<Arc<dyn AnyContextAwareEventListener>>>,
}

impl EventPublisher {
    /// Creates a new event publisher
    ///
    /// # Examples
    ///
    /// ```rust
    /// use verdure_context::EventPublisher;
    ///
    /// let publisher = EventPublisher::new();
    /// ```
    pub fn new() -> Self {
        Self {
            listeners: DashMap::new(),
            context_aware_listeners: DashMap::new(),
        }
    }

    /// Subscribes a context-aware listener to events of type `T`
    ///
    /// Context-aware listeners receive both the event and a reference to the ApplicationContext,
    /// allowing them to interact with the context during event handling.
    ///
    /// # Arguments
    ///
    /// * `listener` - The context-aware event listener to register
    ///
    /// # Examples
    ///
    /// ```rust
    /// use verdure_context::{EventPublisher, ContextInitializedEvent, ContextAwareEventListener, ApplicationContext};
    ///
    /// struct StartupListener;
    ///
    /// impl ContextAwareEventListener<ContextInitializedEvent> for StartupListener {
    ///     fn on_context_event(&self, event: &ContextInitializedEvent, context: &ApplicationContext) {
    ///         println!("Context initialized with {} sources", event.config_sources_count);
    ///         println!("App name: {}", context.get_config("app.name"));
    ///     }
    /// }
    ///
    /// let mut publisher = EventPublisher::new();
    /// publisher.subscribe_context_aware(StartupListener);
    /// ```
    pub fn subscribe_context_aware<
        T: Event + 'static,
        L: ContextAwareEventListener<T> + 'static,
    >(
        &self,
        listener: L,
    ) {
        let type_id = TypeId::of::<T>();
        let typed_listener = Arc::new(TypedContextAwareEventListener::new(listener));

        self.context_aware_listeners
            .entry(type_id)
            .or_insert_with(Vec::new)
            .push(typed_listener);
    }
    ///
    /// # Arguments
    ///
    /// * `listener` - The event listener to register
    ///
    /// # Examples
    ///
    /// ```rust
    /// use verdure_context::{EventPublisher, Event, EventListener};
    /// use std::any::Any;
    ///
    /// #[derive(Debug, Clone)]
    /// struct MyEvent;
    ///
    /// impl Event for MyEvent {
    ///     fn name(&self) -> &'static str { "MyEvent" }
    ///     fn as_any(&self) -> &dyn Any { self }
    ///     fn into_any(self: Box<Self>) -> Box<dyn Any> { self }
    /// }
    ///
    /// struct MyListener;
    ///
    /// impl EventListener<MyEvent> for MyListener {
    ///     fn on_event(&self, _event: &MyEvent) {
    ///         println!("Event received!");
    ///     }
    /// }
    ///
    /// let mut publisher = EventPublisher::new();
    /// publisher.subscribe(MyListener);
    /// ```
    pub fn subscribe<T: Event + 'static, L: EventListener<T> + 'static>(&self, listener: L) {
        let type_id = TypeId::of::<T>();
        let typed_listener = Arc::new(TypedEventListener::new(listener));

        self.listeners
            .entry(type_id)
            .or_insert_with(Vec::new)
            .push(typed_listener);
    }

    /// Publishes an event to all registered listeners with context access
    ///
    /// This method publishes the event to both regular listeners and context-aware listeners.
    /// Context-aware listeners will receive a reference to the ApplicationContext.
    ///
    /// # Arguments
    ///
    /// * `event` - The event to publish
    /// * `context` - Reference to the ApplicationContext
    pub fn publish_with_context<T: Event + 'static>(
        &self,
        event: &T,
        context: &crate::context::ApplicationContext,
    ) {
        let type_id = TypeId::of::<T>();

        // Publish to regular listeners
        if let Some(listeners) = self.listeners.get(&type_id) {
            for listener in listeners.iter() {
                listener.handle_event(event);
            }
        }

        // Publish to context-aware listeners
        if let Some(context_listeners) = self.context_aware_listeners.get(&type_id) {
            for listener in context_listeners.iter() {
                listener.handle_context_event(event, context);
            }
        }
    }
    ///
    /// # Arguments
    ///
    /// * `event` - The event to publish
    ///
    /// # Examples
    ///
    /// ```rust
    /// use verdure_context::{EventPublisher, Event, EventListener};
    /// use std::any::Any;
    ///
    /// #[derive(Debug, Clone)]
    /// struct NotificationEvent {
    ///     message: String,
    /// }
    ///
    /// impl Event for NotificationEvent {
    ///     fn name(&self) -> &'static str { "Notification" }
    ///     fn as_any(&self) -> &dyn Any { self }
    ///     fn into_any(self: Box<Self>) -> Box<dyn Any> { self }
    /// }
    ///
    /// let publisher = EventPublisher::new();
    /// let event = NotificationEvent {
    ///     message: "System startup complete".to_string(),
    /// };
    ///
    /// publisher.publish(&event);
    /// ```
    pub fn publish<T: Event + 'static>(&self, event: &T) {
        let type_id = TypeId::of::<T>();

        if let Some(listeners) = self.listeners.get(&type_id) {
            for listener in listeners.iter() {
                listener.handle_event(event);
            }
        }
    }

    /// Gets the number of listeners for a specific event type
    ///
    /// # Returns
    ///
    /// The number of listeners registered for the event type `T`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use verdure_context::{EventPublisher, Event, EventListener};
    /// use std::any::Any;
    ///
    /// #[derive(Debug, Clone)]
    /// struct CountEvent;
    ///
    /// impl Event for CountEvent {
    ///     fn name(&self) -> &'static str { "CountEvent" }
    ///     fn as_any(&self) -> &dyn Any { self }
    ///     fn into_any(self: Box<Self>) -> Box<dyn Any> { self }
    /// }
    ///
    /// struct CountListener;
    ///
    /// impl EventListener<CountEvent> for CountListener {
    ///     fn on_event(&self, _event: &CountEvent) {}
    /// }
    ///
    /// let mut publisher = EventPublisher::new();
    /// assert_eq!(publisher.listener_count::<CountEvent>(), 0);
    ///
    /// publisher.subscribe(CountListener);
    /// assert_eq!(publisher.listener_count::<CountEvent>(), 1);
    /// ```
    pub fn listener_count<T: Event + 'static>(&self) -> usize {
        let type_id = TypeId::of::<T>();
        self.listeners
            .get(&type_id)
            .map(|listeners| listeners.len())
            .unwrap_or(0)
    }

    /// Removes all listeners for all event types
    ///
    /// # Examples
    ///
    /// ```rust
    /// use verdure_context::EventPublisher;
    ///
    /// let mut publisher = EventPublisher::new();
    /// publisher.clear_all_listeners();
    /// ```
    pub fn clear_all_listeners(&mut self) {
        self.listeners.clear();
    }

    /// Gets statistics about registered listeners
    ///
    /// # Returns
    ///
    /// A map of event type names to listener counts
    ///
    /// # Examples
    ///
    /// ```rust
    /// use verdure_context::EventPublisher;
    ///
    /// let publisher = EventPublisher::new();
    /// let stats = publisher.listener_statistics();
    /// println!("Listener statistics: {:?}", stats);
    /// ```
    pub fn listener_statistics(&self) -> HashMap<String, usize> {
        let mut stats = HashMap::new();

        for entry in self.listeners.iter() {
            let type_id = entry.key();
            let listeners = entry.value();

            // We can't easily get the type name from TypeId, so we'll use the TypeId debug format
            let type_name = format!("{:?}", type_id);
            stats.insert(type_name, listeners.len());
        }

        stats
    }
}

impl Default for EventPublisher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[derive(Debug, Clone)]
    struct TestEvent {
        message: String,
    }

    impl Event for TestEvent {
        fn name(&self) -> &'static str {
            "TestEvent"
        }

        fn as_any(&self) -> &dyn Any {
            self
        }

        fn into_any(self: Box<Self>) -> Box<dyn Any> {
            self
        }
    }

    #[derive(Debug, Clone)]
    struct AnotherEvent {
        value: i32,
    }

    impl Event for AnotherEvent {
        fn name(&self) -> &'static str {
            "AnotherEvent"
        }

        fn as_any(&self) -> &dyn Any {
            self
        }

        fn into_any(self: Box<Self>) -> Box<dyn Any> {
            self
        }
    }

    static TEST_COUNTER: AtomicUsize = AtomicUsize::new(0);

    struct TestListener;

    impl EventListener<TestEvent> for TestListener {
        fn on_event(&self, _event: &TestEvent) {
            TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        }
    }

    struct AnotherListener;

    impl EventListener<AnotherEvent> for AnotherListener {
        fn on_event(&self, _event: &AnotherEvent) {
            TEST_COUNTER.fetch_add(10, Ordering::SeqCst);
        }
    }

    #[test]
    fn test_event_publisher_creation() {
        let publisher = EventPublisher::new();
        assert_eq!(publisher.listener_count::<TestEvent>(), 0);
    }

    #[test]
    fn test_event_subscription() {
        let mut publisher = EventPublisher::new();
        publisher.subscribe(TestListener);

        assert_eq!(publisher.listener_count::<TestEvent>(), 1);
        assert_eq!(publisher.listener_count::<AnotherEvent>(), 0);
    }

    #[test]
    fn test_event_publishing() {
        TEST_COUNTER.store(0, Ordering::SeqCst);

        let mut publisher = EventPublisher::new();
        publisher.subscribe(TestListener);

        let event = TestEvent {
            message: "test".to_string(),
        };

        publisher.publish(&event);

        assert_eq!(TEST_COUNTER.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_multiple_listeners_same_event() {
        TEST_COUNTER.store(0, Ordering::SeqCst);

        let mut publisher = EventPublisher::new();
        publisher.subscribe(TestListener);
        publisher.subscribe(TestListener);

        let event = TestEvent {
            message: "test".to_string(),
        };

        publisher.publish(&event);

        assert_eq!(TEST_COUNTER.load(Ordering::SeqCst), 2);
        assert_eq!(publisher.listener_count::<TestEvent>(), 2);
    }

    #[test]
    fn test_different_event_types() {
        TEST_COUNTER.store(0, Ordering::SeqCst);

        let mut publisher = EventPublisher::new();
        publisher.subscribe(TestListener);
        publisher.subscribe(AnotherListener);

        let test_event = TestEvent {
            message: "test".to_string(),
        };
        let another_event = AnotherEvent { value: 42 };

        publisher.publish(&test_event);
        publisher.publish(&another_event);

        // TestListener adds 1, AnotherListener adds 10
        assert_eq!(TEST_COUNTER.load(Ordering::SeqCst), 11);
    }

    #[test]
    fn test_event_without_listeners() {
        let publisher = EventPublisher::new();
        let event = TestEvent {
            message: "no listeners".to_string(),
        };

        // Should not panic
        publisher.publish(&event);
    }

    #[test]
    fn test_clear_listeners() {
        let mut publisher = EventPublisher::new();
        publisher.subscribe(TestListener);
        publisher.subscribe(AnotherListener);

        assert_eq!(publisher.listener_count::<TestEvent>(), 1);
        assert_eq!(publisher.listener_count::<AnotherEvent>(), 1);

        publisher.clear_all_listeners();

        assert_eq!(publisher.listener_count::<TestEvent>(), 0);
        assert_eq!(publisher.listener_count::<AnotherEvent>(), 0);
    }

    #[test]
    fn test_listener_statistics() {
        let mut publisher = EventPublisher::new();
        publisher.subscribe(TestListener);
        publisher.subscribe(AnotherListener);

        let stats = publisher.listener_statistics();
        assert_eq!(stats.len(), 2);
    }

    #[test]
    fn test_event_trait_methods() {
        let event = TestEvent {
            message: "test".to_string(),
        };

        assert_eq!(event.name(), "TestEvent");
        assert!(event.as_any().is::<TestEvent>());
    }
}
