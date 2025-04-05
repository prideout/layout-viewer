mod app_controller;
mod app_shaders;
mod core;
mod old_core;
mod generate_svg;
mod graphics;
mod hover_effect;
mod rsutils;

#[cfg(not(target_arch = "wasm32"))]
mod app_window;

#[cfg(target_arch = "wasm32")]
mod webui;

#[cfg(not(target_arch = "wasm32"))]
pub use app_window::spawn_window;

pub use old_core::Project;
pub use generate_svg::generate_svg;

#[cfg(target_arch = "wasm32")]
pub use webui::App;

pub use core::*;

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
