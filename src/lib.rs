#![allow(dead_code)]

mod bounds;
mod cells;
mod colors;
mod app_controller;
mod graphics;
mod id_map;
mod layer;
mod project;
mod shaders;
mod string_interner;
mod generate_svg;

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
pub use generate_svg::generate_svg;

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
