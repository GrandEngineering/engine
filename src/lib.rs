use std::{collections::HashMap, fmt::Debug, sync::Arc};

pub trait Task: Debug {
    fn run_hip(&mut self) {
        println!("HIP Runtime not available, falling back to CPU");
        self.run_cpu();
    }
    fn run_cpu(&mut self) {
        panic!("CPU run not Implemented");
    }
    fn run(&mut self, run: Option<Runner>) {
        match run {
            Some(Runner::HIP) => self.run_hip(),
            Some(Runner::CPU) | None => self.run_cpu(),
        }
    }
    fn from_bytes(bytes: &[u8]) -> Self
    where
        Self: Sized;
    fn to_bytes(&self) -> Vec<u8>;
}
#[derive(Debug, Clone, Copy)]
pub enum Runner {
    HIP,
    CPU,
}

pub trait TaskRegistry: Default + Clone {
    fn register(&mut self, task: Arc<dyn Task>, mod_id: String, identifier: String);
    fn get(&self, mod_id: String, identifier: String) -> Option<&dyn Task>;
    fn serialize(&self) -> Vec<u8>;
    fn deserialize(bytes: &[u8]) -> Vec<(String, String)>;
}
#[derive(Default, Clone)]
pub struct EngineTaskRegistry {
    pub tasks: HashMap<(String, String), Arc<dyn Task>>,
}
impl TaskRegistry for EngineTaskRegistry {
    fn register(&mut self, task: Arc<dyn Task>, mod_id: String, identifier: String) {
        // Insert the task into the hashmap with (mod_id, identifier) as the key
        self.tasks.insert((mod_id, identifier), task);
    }
    fn get(&self, mod_id: String, identifier: String) -> Option<&dyn Task> {
        self.tasks.get(&(mod_id, identifier)).map(|t| &**t)
    }
    fn serialize(&self) -> Vec<u8> {
        let keys = self
            .tasks
            .keys()
            .cloned()
            .collect::<Vec<(String, String)>>();
        bincode::serialize(&keys).unwrap()
    }
    fn deserialize(bytes: &[u8]) -> Vec<(String, String)> {
        bincode::deserialize(bytes).unwrap()
    }
}
