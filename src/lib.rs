//
// Copyright (c) 2024 Nathan Fiedler
//
pub mod data;
pub mod domain;
pub mod preso;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use crate::preso::leptos::*;
    console_error_panic_hook::set_once();
    _ = console_log::init_with_level(log::Level::Debug);

    leptos::mount_to_body(App);
}
