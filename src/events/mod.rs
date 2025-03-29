use crate::Identifier;
use crate::api::EngineAPI;
use std::sync::{Arc, Mutex, RwLock};
pub mod admin_auth_event;
pub mod auth_event;
pub mod cgrpc_event;
pub mod start_event;
pub struct Events {}
pub fn ID(namespace: &str, id: &str) -> Identifier {
    (namespace.to_string(), id.to_string())
}

impl Events {
    pub fn init(api: &mut EngineAPI) {
        for (id, tsk) in api.task_registry.tasks.iter() {
            api.task_queue
                .tasks
                .insert(id.clone(), Arc::new(Mutex::new(Vec::new())));
        }
        crate::register_event!(
            api,
            core,
            cgrpc_event,
            crate::events::cgrpc_event::CgrpcEvent {
                cancelled: false,
                handler_id: ("".to_string(), "".to_string()),
                id: ("core".to_string(), "cgrpc_event".to_string()),
                payload: Vec::new(),
                output: Arc::new(RwLock::new(Vec::new()))
            }
        );
        //Register Events to the Default impl for less boilerplate
        crate::register_event!(
            api,
            core,
            start_event,
            crate::events::start_event::StartEvent {
                modules: vec![],
                cancelled: false,
                id: ("core".to_string(), "start_event".to_string())
            }
        );
    }
}
