#![allow(dead_code)]

mod app_controller;
mod app_shaders;
mod core;
mod generate_svg;
mod graphics;
mod rsutils;

#[cfg(not(target_arch = "wasm32"))]
mod app_window;

#[cfg(target_arch = "wasm32")]
mod components;

#[cfg(not(target_arch = "wasm32"))]
pub use app_window::spawn_window;

pub use core::Project;
pub use generate_svg::generate_svg;

#[cfg(target_arch = "wasm32")]
pub use components::App;

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
