use libloading::{Library, Symbol};
use std::any::Any;
use std::collections::HashMap;
use std::mem::ManuallyDrop;
use std::sync::{Arc, RwLock};
use tracing::field::debug;
use tracing::{debug, info};

use crate::api::EngineAPI;
#[derive(Clone, Debug)]
pub struct LibraryInstance {
    dynamicLibrary: Arc<ManuallyDrop<Library>>,
    pub metadata: Arc<LibraryMetadata>,
}
#[derive(Debug, Clone)]
pub struct LibraryMetadata {
    pub mod_id: String,
    pub mod_author: String,
    pub rustc_version: String,
    pub api_version: String,
    pub mod_name: String,
    pub mod_version: String,
    pub mod_description: String,
    pub mod_license: String,
    pub mod_credits: String,
    pub mod_dependencies: Vec<String>,
    pub mod_display_url: String,
    pub mod_issue_tracker: String,
}
impl Default for LibraryMetadata {
    fn default() -> Self {
        Self {
            mod_id: String::new(),
            mod_author: String::new(),
            rustc_version: crate::RUSTC_VERSION.to_string(),
            api_version: crate::GIT_VERSION.to_string(),
            mod_name: String::new(),
            mod_version: String::new(),
            mod_description: String::new(),
            mod_license: String::new(),
            mod_credits: String::new(),
            mod_dependencies: Vec::new(),
            mod_display_url: String::new(),
            mod_issue_tracker: String::new(),
        }
    }
}
#[derive(Default, Clone)]
pub struct LibraryManager {
    pub libraries: HashMap<String, LibraryInstance>,
}

impl LibraryManager {
    pub fn drop(self, api: EngineAPI) {
        drop(api);
        drop(self);
    }
    pub fn register_module(&mut self, path: &str, api: &mut EngineAPI) {
        let (lib, metadata): (Library, LibraryMetadata) = unsafe {
            let library = Library::new(path).unwrap();
            let metadataFN: Symbol<unsafe extern "Rust" fn() -> LibraryMetadata> =
                library.get(b"metadata").unwrap();
            let run: Symbol<unsafe extern "Rust" fn(reg: &mut EngineAPI)> =
                library.get(b"run").unwrap();
            let metadata: LibraryMetadata = metadataFN();
            run(api);
            (library, metadata)
        };

        self.libraries.insert(
            metadata.mod_id.clone(),
            LibraryInstance {
                dynamicLibrary: Arc::new(ManuallyDrop::new(lib)),
                metadata: Arc::new(metadata.clone()),
            },
        );
        debug!(
            "Module {} Loaded, made by {}",
            metadata.mod_name, metadata.mod_author
        )
    }
}
