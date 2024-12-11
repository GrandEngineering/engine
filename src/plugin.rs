use libloading::{Library, Symbol};
use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::api::EngineAPI;

pub struct LibraryInstance {
    dynamicLibrary: Arc<Library>,
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

pub struct LibraryManager {
    libraries: HashMap<String, Arc<LibraryInstance>>,
}

impl LibraryManager {
    fn new() -> Self {
        Self {
            libraries: HashMap::new(),
        }
    }
    fn register_module(path: String, api: &mut EngineAPI) {
        let run: Symbol<unsafe extern "Rust" fn(reg: &mut EngineAPI)>;
        let lib = unsafe {
            let library = Library::new("target/debug/libengine_core.so").unwrap();
            let run: Symbol<unsafe extern "Rust" fn(reg: &mut EngineAPI)> =
                library.get(b"run").unwrap();
            run(api);
            library // Return the library to keep it in scope
        };
        std::mem::forget(lib);
    }
}
