mod bounds;
mod cells;
mod string_interner;
mod project;
mod layer;
mod svg_backend;
mod gl_renderer;
mod gl_scene;
mod gl_geometry;
mod gl_mesh;
mod gl_material;
mod gl_camera;
mod id_map;
mod gl_backend;
mod gl_viewport;

#[cfg(not(target_arch = "wasm32"))]
mod gl_desktop;

#[cfg(not(target_arch = "wasm32"))]
pub use gl_desktop::run_gl_window;

pub use project::{LayoutStats, Project};
pub use gl_scene::Scene;

pub use gl_backend::populate_scene;
pub use svg_backend::generate_svg;

#[cfg(target_arch = "wasm32")]
pub fn run_gl_window() -> anyhow::Result<()> {
    Err(anyhow::anyhow!(
        "OpenGL rendering is not supported in web builds"
    ))
}
