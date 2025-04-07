pub mod core;
pub mod graphics;
pub mod rsutils;

#[cfg(target_arch = "wasm32")]
pub mod webui;

#[cfg(not(target_arch = "wasm32"))]
pub mod cli;

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
