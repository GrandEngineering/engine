use std::{
    any::Any,
    sync::{Arc, RwLock},
};

use crate::{Identifier, api::EngineAPI, event::Event};

use super::{Events, ID};

#[derive(Clone, Debug)]
pub struct AuthEvent {
    pub cancelled: bool,
    pub id: Identifier,
    pub payload: String,
    pub output: Arc<RwLock<bool>>,
}
#[macro_export]
macro_rules! RegisterAuthEventHandler {
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
                $handler_fn(event)
            }
        }
    };
}
impl Events {
    pub fn AuthEvent(api: &mut EngineAPI, payload: String, output: Arc<RwLock<bool>>) {
        api.event_bus.handle(
            ID("core", "auth_event"),
            &mut AuthEvent {
                cancelled: false,
                id: ID("core", "auth_event"),

                payload,
                output,
            },
        );
    }
}

impl Event for AuthEvent {
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
