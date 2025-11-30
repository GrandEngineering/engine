use std::{any::Any, process, sync::Arc};

use tracing::info;

use crate::{
    Identifier,
    api::EngineAPI,
    event::Event,
    plugin::{LibraryManager, LibraryMetadata},
};

use super::{Events, ID};

#[derive(Clone, Debug)]
pub struct StartEvent {
    pub modules: Vec<Arc<LibraryMetadata>>,
    pub cancelled: bool,
    pub id: Identifier,
}

impl Events {
    pub fn StartEvent(api: &mut EngineAPI) {
        let lib_manager = api.lib_manager.clone();
        info!("Started on {}", api.cfg.config_toml.host);
        api.event_bus.handle(
            ID("core", "start_event"),
            &mut StartEvent {
                cancelled: false,
                id: ID("core", "start_event").clone(),
                modules: lib_manager
                    .libraries
                    .values()
                    .cloned()
                    .map(|lib| lib.metadata)
                    .collect(),
            },
        );
    }
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
