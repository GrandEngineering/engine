use std::{collections::HashMap, fmt::Debug, sync::Arc};

use tracing::event as tevent;
use tracing::instrument;
use tracing::Level;
pub mod event;

pub type Identifier = (String, String);
#[derive(Debug, Clone, Default)]
pub struct ModCTX {
    pub mod_id: String,
    pub mod_author: String,
    //pub rustc_version: String,
    pub mod_name: String,
    pub mod_version: String,
    pub mod_description: String,
    pub mod_license: String,
    pub mod_credits: String,
    pub mod_dependencies: Vec<String>,
    pub mod_display_url: String,
    pub mod_issue_tracker: String,
}
impl Clone for Box<dyn Task> {
    fn clone(&self) -> Box<dyn Task> {
        self.clone_box()
    }
}
//pub use engine_derive;

pub trait Task: Debug + Sync + Send {
    fn clone_box(&self) -> Box<dyn Task>;
    #[instrument]
    fn run_hip(&mut self) {
        println!("HIP Runtime not available, falling back to CPU");
        self.run_cpu();
    }
    #[instrument]
    fn run_cpu(&mut self) {
        tevent!(Level::ERROR, "CPU run not Implemented");
        println!("Hi Mom!")
        //panic!("CPU run not Implemented");
    }
    #[instrument]
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
pub trait Registry<T: ?Sized>: Default + Clone {
    fn register(&mut self, registree: Arc<T>, identifier: Identifier);
    fn get(&self, identifier: &Identifier) -> Option<Box<T>>;
}
pub trait VecRegistry<T: ?Sized>: Default + Clone {
    fn register<H>(&mut self, registree: H, identifier: Identifier);
    fn get(&self, identifier: &Identifier) -> Option<Vec<Arc<T>>>;
}
pub trait TaskRegistry: Default + Clone {
    fn register(&mut self, task: Arc<dyn Task>, identifier: Identifier);
    fn get(&self, mod_id: String, identifier: String) -> Option<&dyn Task>;
    fn serialize(&self) -> Vec<u8>;
    fn deserialize(bytes: &[u8]) -> Vec<Identifier>;
}
#[derive(Default, Clone, Debug)]
pub struct EngineTaskRegistry {
    pub tasks: HashMap<Identifier, Arc<dyn Task>>,
}
impl Registry<dyn Task> for EngineTaskRegistry {
    fn register(&mut self, task: Arc<dyn Task>, identifier: Identifier) {
        // Insert the task into the hashmap with (mod_id, identifier) as the key
        self.tasks.insert(identifier, task);
    }

    fn get(&self, identifier: &Identifier) -> Option<Box<dyn Task>> {
        self.tasks.get(identifier).map(|obj| obj.clone_box())
    }
}
