//
// Copyright (c) 2024 Nathan Fiedler
//
use std::env;
use std::path::PathBuf;
use std::sync::LazyLock;

pub mod data;
pub mod domain;
pub mod preso;

#[cfg(test)]
static DEFAULT_DB_PATH: &str = "tmp/test/database";
#[cfg(not(test))]
static DEFAULT_DB_PATH: &str = "tmp/database";

#[cfg(test)]
static DEFAULT_ASSETS_PATH: &str = "tmp/test/blobs";
#[cfg(not(test))]
static DEFAULT_ASSETS_PATH: &str = "tmp/blobs";

// Path to the database files.
pub static DB_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    let path = env::var("DATABASE_PATH").unwrap_or_else(|_| DEFAULT_DB_PATH.to_owned());
    PathBuf::from(path)
});

// Path for uploaded files.
pub static UPLOAD_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    let path = env::var("UPLOAD_PATH").unwrap_or_else(|_| "tmp/uploads".to_owned());
    PathBuf::from(path)
});

pub static ASSETS_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    let path = env::var("ASSETS_PATH").unwrap_or_else(|_| DEFAULT_ASSETS_PATH.to_owned());
    PathBuf::from(path)
});

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use crate::preso::leptos::App;
    console_error_panic_hook::set_once();
    _ = console_log::init_with_level(log::Level::Debug);

    leptos::mount::hydrate_body(App);
}
