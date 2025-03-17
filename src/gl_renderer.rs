use crate::{gl_camera::Camera, gl_viewport::Viewport, Scene};
use glow::*;

pub struct Renderer {
    gl: glow::Context,
    viewport: Viewport,
    clear_color: (f32, f32, f32, f32),
}

impl Renderer {
    pub fn new(gl: glow::Context) -> Self {
        Self {
            gl,
            viewport: Viewport {
                left: 0.0,
                top: 0.0,
                width: 800.0,
                height: 600.0,
            },
            clear_color: (0.1, 0.1, 0.1, 1.0),
        }
    }

    pub fn gl(&self) -> &glow::Context {
        &self.gl
    }

    #[cfg(debug_assertions)]
    pub fn check_gl_error(&self, location: &str) {
        unsafe {
            let error = self.gl.get_error();
            if error != glow::NO_ERROR {
                let error_str = match error {
                    glow::INVALID_ENUM => "GL_INVALID_ENUM",
                    glow::INVALID_VALUE => "GL_INVALID_VALUE",
                    glow::INVALID_OPERATION => "GL_INVALID_OPERATION",
                    glow::INVALID_FRAMEBUFFER_OPERATION => "GL_INVALID_FRAMEBUFFER_OPERATION",
                    glow::OUT_OF_MEMORY => "GL_OUT_OF_MEMORY",
                    glow::STACK_UNDERFLOW => "GL_STACK_UNDERFLOW",
                    glow::STACK_OVERFLOW => "GL_STACK_OVERFLOW",
                    _ => "Unknown GL error",
                };
                log::error!(
                    "OpenGL error at {}: {} (0x{:X})",
                    location,
                    error_str,
                    error
                );
            }
        }
    }

    #[cfg(not(debug_assertions))]
    pub fn check_gl_error(&self, _location: &str) {
        // No-op in release builds
    }

    /// Sets the screen space rectangle in which to draw.
    /// This is the region that the camera's projection quad fits to.
    ///
    /// NOTE: For now we do not bother scissoring to the viewport, which we will
    /// need for features like splitting the screen into multiple viewports.
    pub fn set_viewport(&mut self, viewport: Viewport) {
        self.viewport = viewport;
    }

    pub fn get_viewport(&self) -> Viewport {
        self.viewport
    }

    pub fn set_clear_color(&mut self, r: f32, g: f32, b: f32, a: f32) {
        self.clear_color = (r, g, b, a);
    }

    pub fn get_clear_color(&self) -> (f32, f32, f32, f32) {
        self.clear_color
    }

    pub fn render(&self, scene: &mut Scene, camera: &Camera) {
        unsafe {
            let gl = &self.gl;
            let vp = &self.viewport;

            gl.viewport(
                vp.left as i32,
                vp.top as i32,
                vp.width as i32,
                vp.height as i32,
            );
            let (r, g, b, a) = self.clear_color;
            gl.clear_color(r, g, b, a);
            gl.clear(glow::COLOR_BUFFER_BIT);

            let projection = camera.get_projection_matrix();
            let view_matrix = camera.get_view_matrix();

            for mesh in scene.meshes.values() {
                let material = scene.materials.get_mut(&mesh.material_id).unwrap();
                let geometry = scene.geometries.get_mut(&mesh.geometry_id).unwrap();

                let model_matrix = mesh.matrix;
                material.bind(gl);
                material.set_mat4(&self.gl, "model", &model_matrix);
                material.set_mat4(&self.gl, "view", &view_matrix);
                material.set_mat4(&self.gl, "projection", &projection);

                mesh.draw(gl, material, geometry);
            }
        }
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        // TODO
    }
}
