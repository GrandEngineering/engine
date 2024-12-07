use tracing::Level;

use crate::{
    event::{EngineEventHandlerRegistry, EngineEventRegistry, EventBus},
    task::EngineTaskRegistry,
    ModCTX,
};
use std::{collections::HashMap, sync::Arc};

pub struct EngineAPI {
    pub task_registry: EngineTaskRegistry,
    pub event_bus: EventBus,
    pub modules: HashMap<String, Arc<ModCTX>>,
}
impl Default for EngineAPI {
    fn default() -> Self {
        //Init Logger Here
        tracing_subscriber::FmtSubscriber::builder()
            // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
            // will be written to stdout.
            .with_max_level(Level::INFO)
            // builds the subscriber.
            .init();

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
            modules: HashMap::new(),
        }
    }
}
impl EngineAPI {
    pub fn register_module(&mut self, ctx: ModCTX) -> ModCTX {
        self.modules
            .insert(ctx.clone().mod_id, Arc::new(ctx.clone()));
        ctx
    }
    pub fn setup_logger() {
        tracing_subscriber::FmtSubscriber::builder()
            // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
            // will be written to stdout.
            .with_max_level(Level::INFO)
            // builds the subscriber.
            .init();
    }
}
