use std::{fmt::Debug, sync::Arc};

use crate::Identifier;
use tracing::event as tevent;
use tracing::instrument;
use tracing::Level;

pub trait Task: Debug + Sync + Send + 'static {
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
