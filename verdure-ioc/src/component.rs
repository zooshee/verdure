pub mod factory;

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;
use verdure_core::error::component::ComponentError;

pub type ComponentInstance = Arc<dyn Any + Send + Sync>;

pub enum ComponentScope {
    Singleton,
    Prototype,
}

#[derive(Debug)]
pub struct ComponentDefinition {
    pub type_id: fn() -> TypeId,
    pub type_name: &'static str,
    pub scope: fn() -> ComponentScope,
    pub dependencies: fn() -> Vec<TypeId>,
    pub creator: fn(deps: HashMap<TypeId, ComponentInstance>) -> Result<ComponentInstance, ComponentError>,
}

inventory::collect!(ComponentDefinition);

pub trait ComponentInitializer: Sized {
    type Dependencies;
    fn __new(deps: Self::Dependencies) -> Self;
    fn __scope() -> ComponentScope;
}
