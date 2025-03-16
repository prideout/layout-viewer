mod bounds;
mod cells;
mod gl_backend;
mod gl_camera;
mod gl_geometry;
mod gl_material;
mod gl_mesh;
mod gl_renderer;
mod gl_scene;
mod gl_viewport;
mod id_map;
mod layer;
mod project;
mod string_interner;
mod svg_backend;

#[cfg(not(target_arch = "wasm32"))]
mod gl_window;

#[cfg(not(target_arch = "wasm32"))]
pub use gl_window::run_gl_window;

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
