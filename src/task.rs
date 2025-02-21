use std::{any::Any, fmt::Debug, sync::Arc};

use crate::api::EngineAPI;
use crate::{Identifier, Registry};
use serde::{Deserialize, Serialize};
use tracing::{error, instrument, warn};
pub type StoredTask = (Identifier, Vec<u8>);
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct TaskQueueStorage {
    pub tasks: Vec<Box<StoredTask>>,
}
impl TaskQueueStorage {
    pub fn from_task_queue(task_queue: &TaskQueue) -> Self {
        let mut tasks = Vec::new();
        for task in &task_queue.tasks {
            tasks.push(Box::new((task.get_id(), task.to_bytes())));
        }
        Self { tasks }
    }
}
#[derive(Debug, Default, Clone)]
pub struct TaskQueue {
    pub tasks: Vec<Box<dyn Task>>,
}
impl TaskQueue {
    pub fn from_storage(storage: &TaskQueueStorage, api: &EngineAPI) -> Self {
        let tasks = storage
            .tasks
            .iter()
            .map(|task_bytes| {
                let x: Box<dyn Task> = api.task_registry.get(&task_bytes.0).unwrap_or_else(|| {
                    error!("Failed to convert TaskBytes into Solid Task");
                    panic!("Failed to convert TaskBytes into Solid Task")
                });
                // Assuming you have a way to get the task type and deserialize it
                // This is a placeholder and should be replaced with actual deserialization logic
                let task: Box<dyn Task> = x.from_bytes(&task_bytes.1);
                task
            })
            .collect();
        TaskQueue { tasks }
    }
}

pub trait Task: Debug + Sync + Send {
    fn get_id(&self) -> Identifier;
    fn clone_box(&self) -> Box<dyn Task>;
    #[instrument]
    fn run_hip(&mut self) {
        warn!("HIP Runtime not available, falling back to CPU");
        self.run_cpu();
    }
    #[instrument]
    fn run_cpu(&mut self) {
        error!("CPU run not Implemented");
    }
    #[instrument]
    fn run(&mut self, run: Option<Runner>) {
        match run {
            Some(Runner::HIP) => self.run_hip(),
            Some(Runner::CPU) | None => self.run_cpu(),
        }
    }
    fn to_bytes(&self) -> Vec<u8>;
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
