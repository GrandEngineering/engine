use libloading::{Library, Symbol};
use std::any::Any;
use std::collections::HashMap;
use std::mem::ManuallyDrop;
use std::sync::{Arc, RwLock};

use crate::api::EngineAPI;

pub struct LibraryInstance {
    dynamicLibrary: Arc<ManuallyDrop<Library>>,
    metadata: Arc<LibraryMetadata>,
}
#[derive(Debug, Clone, Default)]
pub struct LibraryMetadata {
    pub mod_id: String,
    pub mod_author: String,
    //pub rustc_version: String,
    //pub api_version:String,
    pub mod_name: String,
    pub mod_version: String,
    pub mod_description: String,
    pub mod_license: String,
    pub mod_credits: String,
    pub mod_dependencies: Vec<String>,
    pub mod_display_url: String,
    pub mod_issue_tracker: String,
}
#[derive(Default, Clone)]
pub struct LibraryManager {
    libraries: HashMap<String, Arc<LibraryInstance>>,
}

impl LibraryManager {
    pub fn register_module(&mut self, path: &str, api: &mut EngineAPI) {
        let run: Symbol<unsafe extern "Rust" fn(reg: &mut EngineAPI)>;
        let metadata: LibraryMetadata;
        let lib = unsafe {
            let library = Library::new(path).unwrap();
            let run: Symbol<unsafe extern "Rust" fn(reg: &mut EngineAPI) -> LibraryMetadata> =
                library.get(b"run").unwrap();
            metadata = run(api);
            library
        };
        self.libraries.insert(
            metadata.mod_id.clone(),
            Arc::new(LibraryInstance {
                dynamicLibrary: Arc::new(ManuallyDrop::new(lib)),
                metadata: Arc::new(metadata),
            }),
        );
    }
}
