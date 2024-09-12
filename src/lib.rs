pub trait Task {
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
    fn from_bytes(&[u8]) -> Self;
    fn to_bytes(&self) -> &[u8];
}
#[derive(Debug, Clone, Copy)]
pub enum Runner {
    HIP,
    CPU,
}
