use std::any::{Any, TypeId};
use std::sync::Arc;

pub trait ComponentFactory {
    fn get_component_by_type_id(&self, type_id: TypeId) -> Option<Arc<dyn Any + Send + Sync>>;

    fn get_component<T: Any + Send + Sync>(&self) -> Option<Arc<T>>;
}
