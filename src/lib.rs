pub enum TaskRunner {
    CPU,
    ROCM,
    CUDA,
}
pub struct Chunk {
    pub start: u128,
    pub end: u128,
}

pub struct Task<TaskInput, TaskOutput> {
    pub task_runner: TaskRunner,
    pub task_id: u128,
    pub task_fn: Box<dyn Fn(TaskInput, &TaskRunner) -> TaskOutput>,
}
impl<TaskInput, TaskOutput> Task<TaskInput, TaskOutput> {
    pub fn execute(&self, input: TaskInput) -> TaskOutput {
        (self.task_fn)(input, &self.task_runner)
    }
}

pub fn isprime(input: Chunk, runner: &TaskRunner) -> u128 {
    input.end * (input.start + input.end)
}
