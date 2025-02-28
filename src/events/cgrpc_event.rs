use std::any::Any;

use crate::{Identifier, api::EngineAPI, event::Event};

use super::{Events, ID};

#[derive(Clone, Debug)]
pub struct CgrpcEvent {
    pub cancelled: bool,
    pub id: Identifier,
    pub handler_id: Identifier,
}
#[macro_export]
macro_rules! RegisterCgrpcEventHandler {
    ($handler:ident,$handler_id:expr,$handler_fn:expr) => {
        use enginelib::events::cgrpc_event::CgrpcEvent;
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
    pub fn CgrpcEvent(api: &mut EngineAPI, handler_id: Identifier) {
        api.event_bus.handle(
            ID("core", "cgrpc_event"),
            &mut CgrpcEvent {
                cancelled: false,
                id: ID("core", "cgrpc_event"),
                handler_id,
            },
        );
    }
}

impl Event for CgrpcEvent {
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
