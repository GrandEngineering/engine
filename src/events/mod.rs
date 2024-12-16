use crate::api::EngineAPI;
use crate::Identifier;
use std::sync::Arc;
pub mod start_event;

pub struct Events {}
pub fn ID(namespace: &str, id: &str) -> Identifier {
    (namespace.to_string(), id.to_string())
}
impl Events {
    pub fn init(api: &mut EngineAPI) {
        //Register Events to the Default impl for less boilerplate
        crate::register_event!(
            api,
            start_event,
            crate::events::start_event::StartEvent {
                modules: vec![],
                id: start_event.clone(),
                cancelled: false,
            }
        );
    }
}
