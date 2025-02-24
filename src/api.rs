use tracing::{debug, Level};

use crate::{
    event::{EngineEventHandlerRegistry, EngineEventRegistry, EventBus},
    task::{Task, TaskQueue},
    Identifier, Registry,
};
pub use bincode::deserialize;
pub use bincode::serialize;
use std::{collections::HashMap, sync::Arc};
pub struct EngineAPI {
    pub task_queue: TaskQueue,
    pub task_registry: EngineTaskRegistry,
    pub event_bus: EventBus,
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
            task_queue: TaskQueue::default(),
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
impl EngineAPI {
    pub fn setup_logger() {
        #[cfg(debug_assertions)]
        tracing_subscriber::FmtSubscriber::builder()
            // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
            // will be written to stdout.
            .with_max_level(Level::DEBUG)
            // builds the subscriber.
            .init();
        #[cfg(not(debug_assertions))]
        tracing_subscriber::FmtSubscriber::builder()
            // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
            // will be written to stdout.
            .with_max_level(Level::INFO)
            // builds the subscriber.
            .init();
    }
}
#[derive(Default, Clone, Debug)]
pub struct EngineTaskRegistry {
    pub tasks: HashMap<Identifier, Arc<dyn Task>>,
}
impl Registry<dyn Task> for EngineTaskRegistry {
    fn register(&mut self, task: Arc<dyn Task>, identifier: Identifier) {
        // Insert the task into the hashmap with (mod_id, identifier) as the key
        debug!(
            "TaskRegistry: Registering task {}.{}",
            identifier.0, identifier.1
        );
        self.tasks.insert(identifier, task);
    }

    fn get(&self, identifier: &Identifier) -> Option<Box<dyn Task>> {
        self.tasks.get(identifier).map(|obj| obj.clone_box())
    }
}
