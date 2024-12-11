use std::{fmt::Debug, sync::Arc};
pub mod api;
pub mod event;
pub mod events;
pub mod macros;
pub mod plugin;
pub mod prelude;
pub mod task;
pub type Identifier = (String, String);

pub trait Registry<T: ?Sized>: Default + Clone {
    fn register(&mut self, registree: Arc<T>, identifier: Identifier);
    fn get(&self, identifier: &Identifier) -> Option<Box<T>>;
}
