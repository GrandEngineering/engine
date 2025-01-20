use std::{any::Any, process, sync::Arc};

use crate::{event::Event, plugin::LibraryMetadata, Identifier};

#[derive(Clone)]
pub struct StartEvent {
    pub modules: Vec<Arc<LibraryMetadata>>,
    pub cancelled: bool,
    pub id: Identifier,
}
#[macro_export]
macro_rules! StartEvent {
    ($lib_manager:expr, $api:expr) => {
        $api.event_bus.handle(
            ID("core", "start_event"),
            &mut events::start_event::StartEvent {
                cancelled: false,
                id: ID("core", "start_event").clone(),
                modules: $lib_manager
                    .libraries
                    .values()
                    .cloned()
                    .map(|lib| lib.metadata)
                    .collect(),
            },
        );
    };
}
impl Event for StartEvent {
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
