#![allow(dead_code)]

use crate::{gl_camera::Camera, gl_viewport::Viewport, Scene};
use glow::*;

pub struct Renderer {
    gl: glow::Context,
    viewport: Viewport,
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
        }
    }

    pub fn set_viewport(&mut self, viewport: Viewport) {
        self.viewport = viewport;
    }

    pub fn get_viewport(&self) -> Viewport {
        self.viewport
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
            gl.clear_color(0.2, 0.4, 0.6, 1.0);
            gl.clear(glow::COLOR_BUFFER_BIT);

            let projection = camera.get_projection_matrix();
            let view_matrix = camera.get_view_matrix();

            for mesh in scene.meshes.values() {
                let material = scene.materials.get_mut(&mesh.material_id).unwrap();
                let geometry = scene.geometries.get_mut(&mesh.geometry_id).unwrap();
                
                let model_matrix = mesh.matrix;
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
