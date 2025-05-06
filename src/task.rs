use std::collections::HashMap;
use std::sync::Mutex;
use std::{fmt::Debug, sync::Arc};

use crate::api::EngineAPI;
use crate::{Identifier, Registry};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::{error, instrument, warn};
#[derive(Clone)]
pub enum TaskState {
    StoredTask(Vec<StoredTask>),
    StoredExecutingTask(Vec<StoredExecutingTask>),
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct StoredTask {
    pub bytes: Vec<u8>,
    pub id: String,
}
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct StoredExecutingTask {
    pub bytes: Vec<u8>,
    pub id: String,
    pub user_id: String,
    pub given_at: DateTime<Utc>,
}
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct TaskQueue {
    pub tasks: HashMap<Identifier, Vec<StoredTask>>,
}
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SolvedTasks {
    pub tasks: HashMap<Identifier, Vec<StoredTask>>,
}
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ExecutingTaskQueue {
    pub tasks: HashMap<Identifier, Vec<StoredExecutingTask>>,
}

pub trait Verifiable {
    fn verify(&self, b: Vec<u8>) -> bool;
}
pub trait Task: Debug + Sync + Send + Verifiable {
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
    fn from_toml(&self, d: String) -> Box<dyn Task>;
    fn to_toml(&self) -> String;
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
