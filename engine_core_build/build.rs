use std::{fs, path::Path};

use oxifs::OxiBuilder;
fn main() {
    #[cfg(debug_assertions)]
    let path = "../target/debug";
    #[cfg(not(debug_assertions))]
    let path = "../target/release";
    let mods_path = Path::new(path).join("mods");
    if !mods_path.exists() {
        fs::create_dir_all(&mods_path).expect("Failed to create mods directory");
    }
    let mut arch = OxiBuilder::init(&format!("{}/mods/engine_core.tar", path));
    arch.add(&format!("{}/libengine_core.so", path), "mod.so");
}
