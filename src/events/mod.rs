use crate::api::EngineAPI;
use crate::event::Event;
use crate::Identifier;
use std::collections::HashMap;
use std::sync::Arc;
pub mod start_event;

pub struct Events {
    events: HashMap<Identifier, Arc<dyn Event>>,
}
pub fn ID(namespace: &str, id: &str) -> Identifier {
    (namespace.to_string(), id.to_string())
}
impl Events {
    pub fn init(api: &mut EngineAPI) -> Self {
        let inst = Self {
            events: HashMap::new(),
        };
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
        inst
    }
}
