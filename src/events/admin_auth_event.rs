use std::{
    any::Any,
    sync::{Arc, RwLock},
};

use crate::{Identifier, api::EngineAPI, event::Event};

use super::{Events, ID};

#[derive(Clone, Debug)]
pub struct AdminAuthEvent {
    pub cancelled: bool,
    pub id: Identifier,
    pub handler_id: Identifier,
    pub payload: Option<String>,
    pub output: Arc<RwLock<bool>>,
}
#[macro_export]
macro_rules! RegisterAdminAuthEventHandler {
    ($handler:ident,$handler_mod_id:ident,$handler_id:ident,$handler_fn:expr) => {
        pub struct $handler;
        impl EventHandler for $handler {
            fn handle(&self, event: &mut dyn Event) {
                let event: &mut CgrpcEvent =
                    <Self as EventCTX<CgrpcEvent>>::get_event::<CgrpcEvent>(event);
                self.handleCTX(event);
            }
        }
        impl EventCTX<CgrpcEvent> for $handler {
            fn handleCTX(&self, event: &mut CgrpcEvent) {
                let id: (String, String) = (
                    stringify!($handler_mod_id).to_string(),
                    stringify!($handler_id).to_string(),
                );
                if (id == event.handler_id) {
                    $handler_fn(event)
                }
            }
        }
    };
}
impl Events {
    pub fn AdminAuthEvent(
        api: &mut EngineAPI,
        handler_id: Identifier,
        payload: Option<String>,
        output: Arc<RwLock<bool>>,
    ) {
        api.event_bus.handle(
            ID("core", "admin_auth_event"),
            &mut AdminAuthEvent {
                cancelled: false,
                id: ID("core", "admin_auth_event"),
                handler_id,
                payload,
                output,
            },
        );
    }
}

impl Event for AdminAuthEvent {
    fn clone_box(&self) -> Box<dyn Event> {
        Box::new(self.clone())
    }

    fn cancel(&mut self) {
        self.cancelled = true;
    }
    fn is_cancelled(&self) -> bool {
        self.cancelled
    }
    fn get_id(&self) -> Identifier {
        self.id.clone()
    }
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
