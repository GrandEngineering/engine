use std::{fmt::Debug, sync::Arc};
pub mod api;
pub mod event;
pub mod events;
pub mod macros;
pub mod prelude;
pub mod task;
pub type Identifier = (String, String);
#[derive(Debug, Clone, Default)]
pub struct ModCTX {
    pub mod_id: String,
    pub mod_author: String,
    //pub rustc_version: String,
    pub mod_name: String,
    pub mod_version: String,
    pub mod_description: String,
    pub mod_license: String,
    pub mod_credits: String,
    pub mod_dependencies: Vec<String>,
    pub mod_display_url: String,
    pub mod_issue_tracker: String,
}
pub trait Registry<T: ?Sized>: Default + Clone {
    fn register(&mut self, registree: Arc<T>, identifier: Identifier);
    fn get(&self, identifier: &Identifier) -> Option<Box<T>>;
}
