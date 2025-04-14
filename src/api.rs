use chrono::{Timelike, Utc};
use sled::Db;
use tokio::{
    spawn,
    sync::RwLock,
    time::{interval, sleep},
};
use tracing::{Level, debug, error, info, instrument};

use crate::{
    Identifier, Registry,
    config::Config,
    event::{EngineEventHandlerRegistry, EngineEventRegistry, EventBus},
    events::Events,
    plugin::LibraryManager,
    task::{ExecutingTaskQueue, SolvedTasks, StoredTask, Task, TaskQueue},
};
pub use bincode::deserialize;
pub use bincode::serialize;
use std::{collections::HashMap, sync::Arc, time::Duration};
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
            cfg: Config::default(),
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
        api.cfg = Config::new();
        Self::init_db(api);
        let mut newLibManager = LibraryManager::default();
        newLibManager.load_modules(api);
        api.lib_manager = newLibManager;
        Events::init(api);
    }
    pub fn init_packer(api: &mut Self) {
        Self::setup_logger();
        let mut newLibManager = LibraryManager::default();
        newLibManager.load_modules(api);
    }
    pub fn init_chron(api: Arc<RwLock<Self>>) {
        let t = api.try_read().unwrap().cfg.config_toml.clean_tasks;
        spawn(clear_sled_periodically(api, t));
    }
    fn init_db(api: &mut EngineAPI) {
        let tasks = api.db.get("tasks");
        let exec_tasks = api.db.get("executing_tasks");
        let solved_tasks = api.db.get("solved_tasks");
        if tasks.is_err() || tasks.unwrap().is_none() {
            let store = bincode::serialize(&api.task_queue.clone()).unwrap();
            api.db.insert("tasks", store).unwrap();
        } else {
            let store = api.db.get("tasks").unwrap().unwrap();
            let res: TaskQueue = bincode::deserialize(&store).unwrap();
            api.task_queue = res;
        }
        if exec_tasks.is_err() || exec_tasks.unwrap().is_none() {
            let store = bincode::serialize(&api.executing_tasks.clone()).unwrap();
            api.db.insert("executing_tasks", store).unwrap();
        } else {
            let store = api.db.get("executing_tasks").unwrap().unwrap();
            let res: ExecutingTaskQueue = bincode::deserialize(&store).unwrap();
            api.executing_tasks = res;
        };
        if solved_tasks.is_err() || solved_tasks.unwrap().is_none() {
            let store = bincode::serialize(&api.solved_tasks.clone()).unwrap();
            api.db.insert("solved_tasks", store).unwrap();
        } else {
            let store = api.db.get("solved_tasks").unwrap().unwrap();
            let res: SolvedTasks = bincode::deserialize(&store).unwrap();
            api.solved_tasks = res;
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
    #[instrument]
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

pub async fn clear_sled_periodically(api: Arc<RwLock<EngineAPI>>, n_minutes: u64) {
    //EngineAPI::setup_logger();
    info!("Sled Cron Job Started");
    let mut interval = interval(Duration::from_secs(n_minutes * 60));
    loop {
        interval.tick().await; // Wait for the interval
        info!("Purging Unsolved Tasks");
        let now = Utc::now().timestamp(); // Current timestamp in seconds
        let mut moved_tasks: Vec<(String, String, StoredTask)> = Vec::new();
        let mut rw_api = api.write().await;
        let db = rw_api.db.clone();
        // Load "executing_tasks"
        if let Ok(Some(tsks)) = db.get("executing_tasks") {
            if let Ok(mut s) = bincode::deserialize::<ExecutingTaskQueue>(&tsks) {
                for ((key1, key2), task_list) in s.tasks.iter_mut() {
                    task_list.retain(|info| {
                        let age = now - info.given_at.timestamp();
                        if age > 3600 {
                            info!("Task {:?} is older than an hour! Moving...", info);
                            moved_tasks.push((
                                key1.clone(),
                                key2.clone(),
                                StoredTask {
                                    id: info.id.clone(),
                                    bytes: info.bytes.clone(),
                                },
                            ));
                            false // Remove old tasks
                        } else {
                            true // Keep tasks that are less than an hour old
                        }
                    });
                }

                // Save updated "executing_tasks"
                if let Ok(updated) = bincode::serialize(&s) {
                    if let Err(e) = db.insert("executing_tasks", updated) {
                        error!("Failed to update executing_tasks in Sled: {:?}", e);
                    }
                }
            }
        }

        // Merge moved tasks into "tasks"
        if !moved_tasks.is_empty() {
            let mut saved_tasks = TaskQueue {
                tasks: HashMap::new(),
            };

            if let Ok(Some(saved_tsks)) = db.get("tasks") {
                if let Ok(existing_tasks) = bincode::deserialize::<TaskQueue>(&saved_tsks) {
                    saved_tasks = existing_tasks;
                }
            }

            // Add moved tasks
            for (key1, key2, task) in moved_tasks {
                saved_tasks
                    .tasks
                    .entry((key1, key2))
                    .or_default()
                    .push(task);
            }

            // Save updated "tasks" queue
            if let Ok(updated) = bincode::serialize(&saved_tasks) {
                if let Err(e) = db.insert("tasks", updated) {
                    error!("Failed to update tasks in Sled: {:?}", e);
                }
            }
        }
        EngineAPI::init_db(&mut rw_api);
    }
}
