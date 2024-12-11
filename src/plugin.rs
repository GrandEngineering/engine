use libloading::Library;
use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

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
