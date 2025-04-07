use crate::graphics::camera::Camera;
use crate::graphics::geometry::Geometry;
use crate::graphics::material::Material;
use crate::graphics::mesh::Mesh;
use crate::graphics::viewport::Viewport;

use bevy_ecs::entity::Entity;
use bevy_ecs::query::QueryState;
use bevy_ecs::system::lifetimeless::Read;
use bevy_ecs::world::World;
use glow::*;

pub struct Renderer {
    gl: glow::Context,
    viewport: Viewport,
    clear_color: (f32, f32, f32, f32),
    mesh_query: Option<QueryState<(Entity, Read<Mesh>)>>,
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
            clear_color: (0.0, 0.0, 0.0, 0.0),
            mesh_query: None,
        }
    }

    pub fn on_new_world(&mut self, world: &mut World) {
        self.mesh_query = Some(world.query());
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

    #[allow(dead_code)]
    pub fn set_clear_color(&mut self, r: f32, g: f32, b: f32, a: f32) {
        self.clear_color = (r, g, b, a);
    }

    pub fn render(&mut self, world: &mut World, camera: &Camera) {
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

            let projection = camera.get_projection_matrix().cast::<f32>();
            let view_matrix = camera.get_view_matrix().cast::<f32>();

            let mesh_query = self.mesh_query.get_or_insert_with(|| world.query());

            let meshes = mesh_query.iter(world).filter_map(|(entity, mesh)| {
                if mesh.visible {
                    Some((entity, mesh.geometry, mesh.material, mesh.render_order))
                } else {
                    None
                }
            });

            let mut meshes: Vec<_> = meshes.collect();

            meshes.sort_by_key(|(_, _, _, render_order)| *render_order);

            for (mesh, geo, mat, _) in meshes {
                let [mesh, mut geo, mut mat] = world.entity_mut([mesh, geo, mat]);
                let mesh = mesh.get::<Mesh>().unwrap();
                let mut geo = geo.get_mut::<Geometry>().unwrap();
                let mut mat = mat.get_mut::<Material>().unwrap();
                mat.bind(gl);
                mat.set_mat4(gl, "view", &view_matrix);
                mat.set_mat4(gl, "projection", &projection);
                mesh.draw(gl, &mut mat, &mut geo);
            }
        }
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        // TODO
    }
}
