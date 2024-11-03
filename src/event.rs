use crate::EngineTaskRegistry;
use crate::Identifier;
use crate::Registry;
use crate::VecRegistry;
use std::any::Any;
use std::collections::HashMap;
use std::process;
use std::sync::Arc;

pub struct EngineAPI {
    pub task_registry: EngineTaskRegistry,
    pub event_bus: EventBus,
}

pub trait EventCTX<C: Event> {
    fn get_event<T: Event + Sized>(event: &mut dyn Event) -> &mut T {
        event.as_any_mut().downcast_mut::<T>().unwrap()
    }
    fn handleCTX(&self, event: &mut C);
}

pub struct EventBus {
    pub event_registry: EngineEventRegistry,
    pub event_handler_registry: EngineEventHandlerRegistry,
}

pub trait Event: Any + Send + Sync {
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
        println!("Handler registered for event ID: {:?}", identifier.clone());
    }
}

impl Registry<dyn Event> for EngineEventRegistry {
    fn register(&mut self, registree: Arc<dyn Event>, identifier: Identifier) {
        self.events.insert(identifier.clone(), registree);
        println!("Event registered with ID: {:?}", identifier.clone());
    }

    fn get(&self, identifier: &Identifier) -> Option<Arc<dyn Event>> {
        self.events.get(identifier).cloned()
    }
}

pub struct OnStartEvent {
    pub modules: Vec<String>,
    pub cancelled: bool,
    pub id: Identifier,
}

impl Event for OnStartEvent {
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

impl Default for EngineAPI {
    fn default() -> Self {
        Self {
            task_registry: EngineTaskRegistry::default(),
            event_bus: EventBus {
                event_registry: EngineEventRegistry {
                    events: HashMap::new(),
                },
                event_handler_registry: EngineEventHandlerRegistry {
                    event_handlers: HashMap::new(),
                },
            },
        }
    }
}

impl EventBus {
    pub fn handle<T: Event + 'static>(&self, id: Identifier, event: &mut T) {
        #[cfg(debug_assertions)]
        println!("Handling event: {:?}", &event.get_id());

        let handlers: &Vec<Arc<dyn EventHandler>> =
            self.event_handler_registry.event_handlers.get(&id).unwrap();

        for handler in handlers {
            let event = event.as_any_mut().downcast_mut::<T>().unwrap();
            handler.handle(event)
        }
    }
}
