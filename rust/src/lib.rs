mod bounds;
mod cells;
mod gl_backend;
mod gl_camera;
mod gl_geometry;
mod gl_material;
mod gl_mesh;
mod gl_renderer;
mod gl_scene;
mod id_map;
mod project;
mod render_layer;
mod string_interner;
mod svg_backend;
mod gl_viewport;

#[cfg(not(target_arch = "wasm32"))]
mod gl_desktop;

pub use svg_backend::generate_svg;
pub use gl_backend::populate_scene;
pub use gl_scene::Scene;

pub use project::{LayoutStats, Project};

#[cfg(not(target_arch = "wasm32"))]
pub use gl_desktop::run_gl_window;

#[cfg(target_arch = "wasm32")]
pub fn run_gl_window() -> anyhow::Result<()> {
    Err(anyhow::anyhow!(
        "OpenGL rendering is not supported in web builds"
    ))
}
