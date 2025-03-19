use std::collections::HashMap;
use std::sync::Mutex;
use std::{fmt::Debug, sync::Arc};

use crate::api::EngineAPI;
use crate::{Identifier, Registry};
use serde::{Deserialize, Serialize};
use tracing::{error, instrument, warn};
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct StoredTask {
    bytes: Vec<u8>,
}
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct TaskQueueStorage {
    pub tasks: HashMap<Identifier, Vec<Box<StoredTask>>>,
}
impl TaskQueueStorage {
    pub fn from_task_queue(task_queue: &TaskQueue) -> Self {
        let mut map: HashMap<Identifier, Vec<Box<StoredTask>>> = HashMap::new();
        for (id, queue) in task_queue.tasks.iter() {
            let task_vec: Vec<Box<StoredTask>> = queue
                .lock()
                .unwrap()
                .iter()
                .filter_map(|task| Some(task.to_bytes()))
                .filter_map(|task_b| Some(StoredTask { bytes: task_b }))
                .filter_map(|task_struct| Some(Box::new(task_struct)))
                .collect();
            map.insert(id.clone(), task_vec);
        }
        Self { tasks: map }
    }
}
#[derive(Debug, Default, Clone)]
pub struct TaskQueue {
    pub tasks: HashMap<Identifier, Arc<Mutex<Vec<Box<dyn Task>>>>>,
}
impl TaskQueue {
    pub fn from_storage(storage: &TaskQueueStorage, api: &EngineAPI) -> Self {
        let mut map: HashMap<(String, String), Arc<Mutex<Vec<Box<dyn Task>>>>> = HashMap::new();
        for (id, tasks) in storage.tasks.iter() {
            let task_vec: Vec<Box<dyn Task>> = tasks
                .iter()
                .filter_map(|task_bytes| match api.task_registry.get(&id) {
                    Some(x) => Some(x.from_bytes(&task_bytes.bytes)),
                    None => {
                        error!(
                            "TaskQueue: Failed to deserialize task {}.{} - invalid data",
                            &id.0, &id.1
                        );
                        None
                    }
                })
                .collect();
            map.insert(id.clone(), Arc::new(Mutex::new(task_vec)));
        }
        TaskQueue { tasks: map }
    }
}

pub trait Task: Debug + Sync + Send {
    fn get_id(&self) -> Identifier;
    fn clone_box(&self) -> Box<dyn Task>;
    #[instrument]
    fn run_hip(&mut self) {
        warn!(
            "Task: HIP runtime not available for {}.{}, falling back to CPU",
            self.get_id().0,
            self.get_id().1
        );
        self.run_cpu();
    }
    #[instrument]
    fn run_cpu(&mut self) {
        error!(
            "Task: CPU implementation missing for {}.{}",
            self.get_id().0,
            self.get_id().1
        );
    }
    #[instrument]
    fn run(&mut self, run: Option<Runner>) {
        match run {
            Some(Runner::HIP) => self.run_hip(),
            Some(Runner::CPU) | None => self.run_cpu(),
        }
    }
    fn to_bytes(&self) -> Vec<u8>;
    #[allow(clippy::wrong_self_convention)]
    fn from_bytes(&self, bytes: &[u8]) -> Box<dyn Task>;
}

#[derive(Debug, Clone, Copy)]
pub enum Runner {
    HIP,
    CPU,
}

pub trait TaskRegistry: Default + Clone {
    fn register(&mut self, task: Arc<dyn Task>, identifier: Identifier);
    fn get(&self, mod_id: String, identifier: String) -> Option<&dyn Task>;
    fn serialize(&self) -> Vec<u8>;
    fn deserialize(bytes: &[u8]) -> Vec<Identifier>;
}

impl Clone for Box<dyn Task> {
    fn clone(&self) -> Box<dyn Task> {
        self.clone_box()
    }
}
