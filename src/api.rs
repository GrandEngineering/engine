use tracing::{Level, debug};

use crate::{
    Identifier, Registry,
    config::Config,
    event::{EngineEventHandlerRegistry, EngineEventRegistry, EventBus},
    events::Events,
    plugin::LibraryManager,
    task::{ExecutingTaskQueue, SolvedTasks, Task, TaskQueue},
};
pub use bincode::deserialize;
pub use bincode::serialize;
use std::{collections::HashMap, sync::Arc};
pub struct EngineAPI {
    pub cfg: Config,
    pub task_queue: TaskQueue,
    pub executing_tasks: ExecutingTaskQueue,
    pub solved_tasks: SolvedTasks,
    pub task_registry: EngineTaskRegistry,
    pub event_bus: EventBus,
    pub db: sled::Db,
    pub lib_manager: LibraryManager,
}

impl Default for EngineAPI {
    fn default() -> Self {
        Self {
            cfg: Config::new(),
            task_queue: TaskQueue::default(),
            db: sled::open("engine_db").unwrap(),
            lib_manager: LibraryManager::default(),
            task_registry: EngineTaskRegistry::default(),
            event_bus: EventBus {
                event_registry: EngineEventRegistry {
                    events: HashMap::new(),
                },
                event_handler_registry: EngineEventHandlerRegistry {
                    event_handlers: HashMap::new(),
                },
            },
            solved_tasks: SolvedTasks::default(),
            executing_tasks: ExecutingTaskQueue::default(),
        }
    }
}
impl EngineAPI {
    pub fn test_default() -> Self {
        Self {
            cfg: Config::new(),
            task_queue: TaskQueue::default(),
            db: sled::Config::new()
                .temporary(true)
                .flush_every_ms(None)
                .open()
                .unwrap(),
            lib_manager: LibraryManager::default(),
            task_registry: EngineTaskRegistry::default(),
            event_bus: EventBus {
                event_registry: EngineEventRegistry {
                    events: HashMap::new(),
                },
                event_handler_registry: EngineEventHandlerRegistry {
                    event_handlers: HashMap::new(),
                },
            },
            solved_tasks: SolvedTasks::default(),
            executing_tasks: ExecutingTaskQueue::default(),
        }
    }
    pub fn init(api: &mut Self) {
        Self::setup_logger();
        Events::init(api);
        let mut newLibManager = LibraryManager::default();
        newLibManager.load_modules(api);
        api.lib_manager = newLibManager;
    }
    fn init_db(api: &mut EngineAPI) {
        let tasks = api.db.get("tasks");
        let exec_tasks = api.db.get("executing_tasks");
        if tasks.is_err() {
            let store = bincode::serialize(&api.task_queue.clone()).unwrap();
            api.db.insert("tasks", store).unwrap();
        } else {
            let store = api.db.get("tasks").unwrap().unwrap();
            let res: TaskQueue = bincode::deserialize(&store).unwrap();
            api.task_queue = res;
        }
        if exec_tasks.is_err() {
            let store = bincode::serialize(&api.executing_tasks.clone()).unwrap();
            api.db.insert("executing_tasks", store).unwrap();
        } else {
            let store = api.db.get("executing_tasks").unwrap().unwrap();
            let res: ExecutingTaskQueue = bincode::deserialize(&store).unwrap();
            api.executing_tasks = res;
        };
    }
    pub fn init_dev(api: &mut Self) {
        Self::setup_logger();
        Events::init(api);
        let mut newLibManager = LibraryManager::default();
        newLibManager
            .load_library("./target/release/libengine_core.so", api)
            .unwrap();
        api.lib_manager = newLibManager;
    }
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
