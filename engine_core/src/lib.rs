use enginelib::{EngineTaskRegistry, Task, TaskRegistry};
use std::sync::Arc;
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
pub fn run(reg: &mut EngineTaskRegistry) {
    let mod_id = "namespace".to_string();
    let task_id = "fib".to_string();
    reg.register(
        Arc::new(FibTask::default()),
        mod_id.clone(),
        task_id.clone(),
    );
    println!("Registered task: {}:{}", &mod_id, &task_id);
}
