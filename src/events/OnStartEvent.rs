#[derive(Clone)]
pub struct OnStartEvent {
    pub modules: Vec<Arc<ModCTX>>,
    pub cancelled: bool,
    pub id: Identifier,
}

impl Event for OnStartEvent {
    fn clone_box(&self) -> Box<dyn Event> {
        Box::new(self.clone())
    }

    fn cancel(&mut self) {
        self.cancelled = true;
        process::exit(0)
    }
    fn is_cancelled(&self) -> bool {
        self.cancelled
    }
    fn get_id(&self) -> Identifier {
        self.id.clone()
    }
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
