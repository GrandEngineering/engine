use crate::plugin::LibraryInstance;
use crate::Identifier;
use crate::Registry;
use std::any::Any;
use std::collections::HashMap;
use std::process;
use std::sync::Arc;
pub use tracing::{debug, error, event, info, warn};
// The Actual Fuck
// this fucking piece of god given code saves so much time

pub trait EventCTX<C: Event>: EventHandler {
    fn get_event<T: Event + Sized>(event: &mut dyn Event) -> &mut T {
        event.as_any_mut().downcast_mut::<T>().unwrap()
    }
    fn handle(&self, event: &mut dyn Event) {
        let event: &mut C = <Self as EventCTX<C>>::get_event::<C>(event);
        self.handleCTX(event);
    }
    #[allow(non_snake_case)]
    fn handleCTX(&self, event: &mut C);
}

pub struct EventBus {
    pub event_registry: EngineEventRegistry,
    pub event_handler_registry: EngineEventHandlerRegistry,
}

pub trait Event: Any + Send + Sync {
    fn clone_box(&self) -> Box<dyn Event>;
    fn cancel(&mut self);
    fn is_cancelled(&self) -> bool;
    fn get_id(&self) -> Identifier;

    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub trait EventHandler: Any + Send + Sync {
    fn handle(&self, event: &mut dyn Event);
}

#[derive(Default, Clone)]
pub struct EngineEventRegistry {
    pub events: HashMap<Identifier, Arc<dyn Event>>,
}

#[derive(Clone, Default)]
pub struct EngineEventHandlerRegistry {
    pub event_handlers: HashMap<Identifier, Vec<Arc<dyn EventHandler>>>,
}

impl EngineEventHandlerRegistry {
    pub fn register_handler<H: EventHandler + Send + Sync + 'static>(
        &mut self,
        handler: H,
        identifier: Identifier,
    ) {
        let handler = Arc::new(handler);
        let handlers = self.event_handlers.entry(identifier.clone()).or_default();
        handlers.push(handler);
        debug!(
            "Event Handler registered for event ID: {:?}",
            identifier.clone()
        );
    }
}
impl Clone for Box<dyn Event> {
    fn clone(&self) -> Box<dyn Event> {
        self.clone_box()
    }
}
impl Registry<dyn Event> for EngineEventRegistry {
    fn register(&mut self, registree: Arc<dyn Event>, identifier: Identifier) {
        self.events.insert(identifier.clone(), registree);
        debug!("Event registered with ID: {:?}", identifier.clone());
    }

    fn get(&self, identifier: &Identifier) -> Option<Box<dyn Event>> {
        self.events.get(identifier).map(|obj| obj.clone_box())
    }
}
#[derive(Clone)]
pub struct OnStartEvent {
    pub modules: Vec<Arc<LibraryInstance>>,
    pub cancelled: bool,
    pub id: Identifier,
}

impl Event for OnStartEvent {
    fn clone_box(&self) -> Box<dyn Event> {
        Box::new(self.clone())
    }

    fn cancel(&mut self) {
        self.cancelled = true;
        process::exit(0)
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

impl EventBus {
    pub fn handle<T: Event>(&self, id: Identifier, event: &mut T) {
        debug!("Handling events: {:?}", &event.get_id());
        let handlers: Option<&Vec<Arc<dyn EventHandler>>> =
            self.event_handler_registry.event_handlers.get(&id);

        if let Some(handlers) = handlers {
            for handler in handlers {
                if let Some(event) = event.as_any_mut().downcast_mut::<T>() {
                    handler.handle(event)
                } else {
                    error!("Failed to downcast event during handling");
                }
            }
        } else {
            debug!("No EventHandlers subscribed to {:?}:{:?}", id.0, id.1)
        }
    }
}
