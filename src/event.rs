use crate::EngineTaskRegistry;
use crate::Identifier;
use crate::Registry;
use std::any::Any;
use std::collections::HashMap;
use std::process;
use std::sync::Arc;

pub struct EngineAPI {
    pub task_registry: EngineTaskRegistry,
    pub event_bus: EventBus,
}
#[macro_export]
macro_rules! BuildEventHandler {
    ($handler:ident,$event:ty, $handler_fn:expr) => {
        pub struct $handler;
        impl EventHandler for $handler {
            fn handle(&self, event: &mut dyn Event) {
                let event: &mut $event = <Self as EventCTX<$event>>::get_event::<$event>(event);
                self.handleCTX(event);
            }
        }
        impl EventCTX<$event> for $handler {
            fn handleCTX(&self, event: &mut $event) {
                $handler_fn(event)
            }
        }
    };
    ($handler:ident,$event:ty,$mod_ctx:expr, $handler_fn:expr) => {
        pub struct $handler {
            mod_ctx: ModCTX,
        };
        impl $handler {
            pub fn new(mod_ctx: ModCTX) -> Self {
                Self { mod_ctx }
            }
        }
        impl EventHandler for $handler {
            fn handle(&self, event: &mut dyn Event) {
                let event: &mut $event = <Self as EventCTX<$event>>::get_event::<$event>(event);
                self.handleCTX(event);
            }
        }
        impl EventCTX<$event> for $handler {
            fn handleCTX(&self, event: &mut $event) {
                let mod_ctx: &ModCTX = &self.mod_ctx;
                $handler_fn(event, mod_ctx.clone())
            }
        }
    };
}

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
        println!("Handler registered for event ID: {:?}", identifier.clone());
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
        println!("Event registered with ID: {:?}", identifier.clone());
    }

    fn get(&self, identifier: &Identifier) -> Option<Box<dyn Event>> {
        self.events.get(identifier).map(|obj| obj.clone_box())
    }
}
#[derive(Clone)]
pub struct OnStartEvent {
    pub modules: Vec<String>,
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
