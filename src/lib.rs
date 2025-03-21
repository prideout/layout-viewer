#![allow(dead_code)]

mod bounds;
mod cells;
mod colors;
mod controller;
mod gl_backend;
mod graphics;
mod id_map;
mod layer;
mod project;
mod string_interner;
mod svg_backend;

#[cfg(not(target_arch = "wasm32"))]
mod gl_window;

#[cfg(target_arch = "wasm32")]
mod pages;

#[cfg(target_arch = "wasm32")]
mod resize_observer;

#[cfg(target_arch = "wasm32")]
mod components;

#[cfg(not(target_arch = "wasm32"))]
pub use gl_window::spawn_window;

pub use project::Project;
pub use gl_backend::populate_scene;
pub use svg_backend::generate_svg;

#[cfg(target_arch = "wasm32")]
pub use components::app::App;

/// Returns a timestamp in milliseconds.
#[cfg(target_arch = "wasm32")]
#[macro_export]
macro_rules! performance_now {
    () => {
        web_sys::window()
            .and_then(|w| w.performance().and_then(|f| Some(f.now())))
            .unwrap_or(0.0)
    };
}
