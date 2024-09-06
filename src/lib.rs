pub trait Task {
    fn run_hip(&mut self) {
        println!("HIP Runtime not availible running with the CPU");
        self.run_cpu();
    }
    fn run_cpu(&mut self) {
        panic!("CPU run not Implemented");
    }
    fn run(&mut self, run: Option<Runner>) {
        match run {
            Some(Runner::HIP) => self.run_hip(),
            Some(Runner::CPU) => self.run_cpu(),
            None => self.run_cpu(),
        }
    }
}
pub enum Runner {
    HIP,
    CPU,
}
