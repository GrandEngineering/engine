use auth_event::AuthEvent;

use crate::api::{self, EngineAPI};
use crate::{Identifier, RegisterAdminAuthEventHandler, RegisterAuthEventHandler};
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
    pub fn init_auth(api: &mut EngineAPI) {
        RegisterAuthEventHandler!(AuthHandler, |event: &mut AuthEvent| {
            *event.output.write().unwrap() = true;
        });
        api.event_bus
            .event_handler_registry
            .register_handler(AuthHandler {}, ID("core", "auth_event"));
        let token = api.cfg.config_toml.cgrpc_token.clone();
        if let Some(token) = token {
            RegisterAdminAuthEventHandler!(
                AdminAuthHandler,
                String,
                |event: &mut AdminAuthEvent, mod_ctx: &Arc<String>| {
                    let token: &Arc<String> = mod_ctx;
                    let token: String = token.as_str().to_string();
                    if token == event.payload {
                        *event.output.write().unwrap() = true;
                    }
                }
            );
            api.event_bus.event_handler_registry.register_handler(
                AdminAuthHandler {
                    mod_ctx: Arc::new(token),
                },
                ID("core", "admin_auth_event"),
            );
        } else {
            RegisterAdminAuthEventHandler!(AdminAuthHandler, |event: &mut AdminAuthEvent| {
                *event.output.write().unwrap() = true;
            });
            api.event_bus
                .event_handler_registry
                .register_handler(AdminAuthHandler, ID("core", "admin_auth_event"));
        }
    }
    pub fn init(api: &mut EngineAPI) {
        for (id, tsk) in api.task_registry.tasks.iter() {
            api.task_queue.tasks.entry(id.clone()).or_default();
            api.executing_tasks.tasks.entry(id.clone()).or_default();
            api.solved_tasks.tasks.entry(id.clone()).or_default();
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
