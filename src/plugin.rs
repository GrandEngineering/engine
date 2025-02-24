use crate::api::EngineAPI;
use libloading::{Library, Symbol};
use oxifs::OxiFS;
use serde::{Deserialize, Serialize};
use std::mem::ManuallyDrop;
use std::sync::Arc;
use std::{collections::HashMap, fs};
use tracing::field::debug;
use tracing::{debug, error, info};
#[derive(Clone, Debug)]
pub struct LibraryInstance {
    dynamicLibrary: Arc<ManuallyDrop<Library>>,
    pub metadata: Arc<LibraryMetadata>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub mod_dependencies: Vec<LibraryDependency>,
    pub mod_display_url: String,
    pub mod_issue_tracker: String,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LibraryDependency {
    pub mod_git_repo: String,
    pub mod_git_commit: String,
    pub mod_id: String,
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
    pub fn load_modules(&mut self, api: &mut EngineAPI) {
        //get all files in ./mods
        let dir_path = "./mods"; // Target directory
        let mut files: Vec<String> = Vec::new();

        if let Ok(entries) = fs::read_dir(dir_path) {
            for entry in entries.filter_map(Result::ok) {
                let path = entry.path();
                if path.is_file() {
                    if let Some(extension) = path.extension() {
                        if extension == "tar" {
                            if let Some(stem) = path.file_stem() {
                                if stem.to_string_lossy().ends_with(".rustforge") {
                                    files.push(path.display().to_string());
                                }
                            }
                        }
                    }
                }
            }
        } else {
            eprintln!("Error reading directory: {}", dir_path);
        }
        for file in files {
            self.load_module(&file, api);
        }
    }
    pub fn load_module(&mut self, path: &str, api: &mut EngineAPI) {
        info!("Loading module {}", path);
        let fs = OxiFS::new(path);

        let tmp_path = fs.tempdir.path();
        #[cfg(unix)]
        let library_path = tmp_path.join("mod.so");
        #[cfg(windows)]
        let library_path = tmp_path.join("mod.so");
        if let Some(lib_path_str) = library_path.to_str() {
            self.load_library(lib_path_str, api);
        } else {
            info!("Invalid library path for module: {}", path);
        }
        std::mem::forget(fs);
    }

    pub fn load_library(&mut self, path: &str, api: &mut EngineAPI) -> Result<(), String> {
        debug!("Loading library {}", path);
        let (lib, metadata): (Library, LibraryMetadata) = match unsafe {
            Library::new(path)
                .map_err(|e| error!("Failed to load library: {e}"))
                .and_then(|library| {
                    let metadataFN: Symbol<unsafe extern "Rust" fn() -> LibraryMetadata> = library
                        .get(b"metadata")
                        .map_err(|e| error!("Failed to load metadata: {e}"))?;
                    let metadata: LibraryMetadata = metadataFN();
                    Ok((library, metadata))
                })
        } {
            Ok(result) => result,
            Err(err) => {
                info!("Failed to load module at {:#?}: {:#?}", path, err);
                return Err("Failed to load module".to_string());
            }
        };
        if metadata.api_version != crate::GIT_VERSION
            && metadata.rustc_version != crate::RUSTC_VERSION
        {
            let err = format!(
                            "Module version mismatch - Lib API: {}, Engine API: {}, Lib Rustc: {}, Engine Rustc: {}",
                            metadata.api_version,
                            crate::GIT_VERSION,
                            metadata.rustc_version,
                            crate::RUSTC_VERSION
                        );
            error!("{}", err);
            return Err(err);
        };

        let res = unsafe {
            lib.get(b"run")
                .map_err(|e| {
                    error!("Failed to get run symbol: {:#?}", e);
                })
                .map(
                    |run: Symbol<unsafe extern "Rust" fn(reg: &mut EngineAPI)>| {
                        run(api);
                    },
                )
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
        );
        Ok(())
    }
}
