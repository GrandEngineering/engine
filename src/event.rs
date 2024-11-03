use crate::EngineTaskRegistry;
use crate::Identifier;
use crate::Registry;
use crate::VecRegistry;
use std::collections::HashMap;
use std::process;
use std::sync::Arc;
pub struct EngineAPI {
    pub task_registry: EngineTaskRegistry,
    pub event_bus: EventBus,
}

pub struct EventBus {
    pub event_registry: EngineEventRegistry,
    pub event_handler_registry: EngineEventHandlerRegistry,
}

pub trait Event {
    fn cancel(&mut self);
    fn is_cancelled(&self) -> bool;
    fn get_id(&self) -> Identifier;
}

pub trait EventHandler {
    fn handle(&self, event: &mut Arc<dyn Event>);
}

#[derive(Default, Clone)]
pub struct EngineEventRegistry {
    pub events: HashMap<Identifier, Arc<dyn Event>>,
}

#[derive(Clone, Default)]
pub struct EngineEventHandlerRegistry {
    pub event_handlers: HashMap<Identifier, Vec<Arc<dyn EventHandler>>>,
}

impl VecRegistry<dyn EventHandler> for EngineEventHandlerRegistry {
    fn get(&self, identifier: &Identifier) -> Option<Vec<Arc<dyn EventHandler>>> {
        self.event_handlers.get(identifier).cloned()
    }
    fn register(&mut self, registree: Arc<dyn EventHandler>, identifier: Identifier) {
        let handlers = self.event_handlers.get(&identifier);
        #[cfg(debug_assertions)]
        println!("Registered Event Handler");
        if handlers.is_some() {
            self.event_handlers
                .get(&identifier)
                .unwrap()
                .clone()
                .push(registree);
        } else {
            self.event_handlers.insert(identifier, vec![registree]);
        }
    }
}

impl Registry<dyn Event> for EngineEventRegistry {
    fn register(&mut self, registree: Arc<dyn Event>, identifier: Identifier) {
        self.events.insert(identifier, registree);
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
    pub fn handle(&self, id: Identifier, event: &mut Arc<dyn Event>) {
        #[cfg(debug_assertions)]
        println!("Handling Event: {}:{}", id.0, id.1);
        let handler = self.event_handler_registry.get(&id);
        if handler.is_none() {
            #[cfg(debug_assertions)]
            println!("Empty Handler Vec");
            return;
        }
        for h in handler.unwrap() {
            h.handle(event);
        }
    }
    pub fn handle_default(&self, id: Identifier) {
        let event = self.event_registry.get(&id);
        self.handle(id, &mut event.unwrap());
    }
}
