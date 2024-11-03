use enginelib::event::{Event, EventCTX, EventHandler};
use enginelib::{event, event::OnStartEvent, Registry, Task, VecRegistry};
use std::sync::Arc;
use std::{collections::HashMap, fmt::Debug, process};
#[derive(Debug, Clone, Copy, Default)]
pub struct FibTask {
    pub iter: u64,
    pub result: u64,
}
impl Task for FibTask {
    fn run_cpu(&mut self) {
        let mut a = 0;
        let mut b = 1;
        for _ in 0..self.iter {
            let tmp = a;
            a = b;
            b += tmp;
        }
        self.result = a;
    }
    fn from_bytes(bytes: &[u8]) -> Self {
        let iter = u64::from_le_bytes(bytes[0..8].try_into().unwrap());
        let result = u64::from_le_bytes(bytes[8..16].try_into().unwrap());
        Self { iter, result }
    }
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(16);
        bytes.extend_from_slice(&self.iter.to_le_bytes());
        bytes.extend_from_slice(&self.result.to_le_bytes());
        bytes
    }
}
#[no_mangle]
pub fn run(api: &mut event::EngineAPI) {
    let mod_id = "namespace".to_string();
    let task_id = "fib".to_string();

    api.task_registry.register(
        Arc::new(FibTask::default()),
        (mod_id.clone(), task_id.clone()),
    );
    api.event_bus.event_handler_registry.register_handler(
        OnStartEventHandler,
        ("core".to_string(), "onstartevent".to_string()),
    );
    println!("Registered task: {}:{}", &mod_id, &task_id);
    //OnStartEventHandler.ha
    //(namespace,event_id)
}
struct OnStartEventHandler;
impl EventHandler for OnStartEventHandler {
    fn handle(&self, event: &mut dyn Event) {
        let event: &mut OnStartEvent =
            <OnStartEventHandler as EventCTX<OnStartEvent>>::get_event::<OnStartEvent>(event);
        self.handleCTX(event);
    }
}
impl EventCTX<OnStartEvent> for OnStartEventHandler {
    fn handleCTX(&self, event: &mut OnStartEvent) {
        println!("here mom!");
    }
}
