mod bounds;
mod cells;
mod string_interner;
mod project;
mod render_layer;
mod svg_backend;
mod vec2d;

#[cfg(not(target_arch = "wasm32"))]
mod gl_desktop;

pub use project::{LayoutStats, Project};
#[cfg(not(target_arch = "wasm32"))]
pub use gl_desktop::run_gl_window;

#[cfg(target_arch = "wasm32")]
pub fn run_gl_window() -> anyhow::Result<()> {
    Err(anyhow::anyhow!("OpenGL rendering is not supported in web builds"))
}
