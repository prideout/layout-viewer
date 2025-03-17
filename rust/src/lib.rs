mod bounds;
mod cells;
pub mod controller;
mod gl_backend;
pub mod gl_camera;
pub mod gl_geometry;
pub mod gl_material;
pub mod gl_mesh;
pub mod gl_renderer;
pub mod gl_scene;
pub mod gl_viewport;
mod id_map;
mod layer;
mod project;
mod string_interner;
mod svg_backend;

#[cfg(not(target_arch = "wasm32"))]
pub use gl_window::spawn_window;

#[cfg(not(target_arch = "wasm32"))]
pub mod gl_window;

pub use gl_scene::Scene;
pub use project::Project;

pub use gl_backend::populate_scene;
pub use svg_backend::generate_svg;

#[cfg(target_arch = "wasm32")]
pub fn run_gl_window() -> anyhow::Result<()> {
    Err(anyhow::anyhow!(
        "OpenGL rendering is not supported in web builds"
    ))
}
