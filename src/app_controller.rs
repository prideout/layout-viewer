use bevy_ecs::entity::Entity;
use bevy_ecs::world::World;

use crate::app_shaders::FRAGMENT_SHADER;
use crate::app_shaders::VERTEX_SHADER;
use crate::graphics::BlendMode;
use crate::graphics::Camera;
use crate::graphics::Geometry;
use crate::graphics::Material;
use crate::graphics::Mesh;
use crate::graphics::Renderer;
use crate::graphics::Viewport;
use crate::hover_effect::HoverEffect;
use crate::hover_effect::HoverParams;
use crate::Project;

type Point3 = nalgebra::Point3<f64>;

/// Encapsulates high-level application logic common to all platforms.
pub struct AppController {
    window_size: (u32, u32),
    renderer: Renderer,
    camera: Camera,
    world: World,
    is_dragging: bool,
    last_mouse_pos: Option<(u32, u32)>,
    zoom_speed: f64,
    needs_render: bool,
    project: Option<Project>,
    hover_effect: HoverEffect,
    layer_material: Entity,
}

pub enum Theme {
    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    Light,
    Dark,
}

#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
impl AppController {
    pub fn new(renderer: Renderer, physical_width: u32, physical_height: u32) -> Self {
        let camera = Camera::new(Point3::new(0.0, 0.0, 0.0), 128.0, 128.0, -1.0, 1.0);

        let mut world = World::new();

        let hover_effect = HoverEffect::new(&mut world);

        let layer_material_component = Material::new(VERTEX_SHADER, FRAGMENT_SHADER);
        let layer_material = world.spawn(layer_material_component).id();

        Self {
            window_size: (physical_width, physical_height),
            renderer,
            camera,
            world,
            is_dragging: false,
            last_mouse_pos: None,
            zoom_speed: 0.05,
            needs_render: true,
            project: None,
            hover_effect,
            layer_material,
        }
    }

    pub fn get_mesh_for_layer_mut(&mut self, layer_index: usize) -> &mut Mesh {
        let layer = &self.project.as_ref().unwrap().layers()[layer_index];
        let id = layer.mesh.unwrap();
        self.world.get_mut::<Mesh>(id).unwrap().into_inner()
    }

    pub fn set_project(&mut self, mut project: Project) {
        let bounds = project.bounds();
        self.camera.fit_to_bounds(self.window_size, bounds);

        let meshes = project.create_meshes(&mut self.world, self.layer_material);

        self.hover_effect.set_render_order(&mut self.world, 9999);

        for (i, layer) in project.layers_mut().iter_mut().enumerate() {
            layer.mesh = Some(meshes[i]);
        }

        self.project = Some(project);
    }

    pub fn handle_mouse_press(&mut self, x: u32, y: u32) {
        self.is_dragging = true;
        self.last_mouse_pos = Some((x, y));
    }

    pub fn handle_mouse_release(&mut self) {
        self.is_dragging = false;
        self.last_mouse_pos = None;
    }

    pub fn handle_mouse_move(&mut self, x: u32, y: u32) {
        if self.is_dragging {
            if let Some((last_x, last_y)) = self.last_mouse_pos {
                let p1 = self.screen_to_world(x, y);
                let p0 = self.screen_to_world(last_x, last_y);
                let dx = p1.0 - p0.0;
                let dy = p1.1 - p0.1;

                let mut pos = self.camera.position;
                pos.x -= dx;
                pos.y -= dy;
                self.camera.position = pos;
                self.render();
            }
            self.last_mouse_pos = Some((x, y));
        }

        // Convert screen coordinates to world space
        let (world_x, world_y) = self.screen_to_world(x, y);

        // Temporarily take the project to avoid borrowing issues
        let Some(project) = self.project.take() else {
            return;
        };

        if let Some(hit) = project.pick_cell(world_x, world_y) {
            if !self.hover_effect.contains(&hit) {
                self.hover_effect.show(HoverParams {
                    selection: hit,
                    project: &project,
                    world: &mut self.world,
                    gl: self.renderer.gl(),
                });
                self.render();
            }
        } else if self.hover_effect.is_visible() {
            self.hover_effect.hide(&mut self.world);
            self.render();
        }

        self.project = Some(project);
    }

    pub fn handle_mouse_wheel(&mut self, x: u32, y: u32, delta: f64) {
        // Ignore very small deltas that might be touchpad bounce
        const MIN_DELTA: f64 = 0.01;
        if delta.abs() < MIN_DELTA {
            return;
        }

        // Convert screen coordinates to world space before zoom
        let (world_x, world_y) = self.screen_to_world(x, y);

        // Calculate zoom factor (positive delta = zoom in, negative = zoom out)
        // Clamp delta to avoid extreme zoom changes
        let clamped_delta = delta.clamp(-1.0, 1.0);
        let zoom_factor = if clamped_delta > 0.0 {
            1.0 - self.zoom_speed
        } else {
            1.0 + self.zoom_speed
        };

        // Update camera size (zoom)
        self.camera.width *= zoom_factor;
        self.camera.height *= zoom_factor;

        // Convert the same screen coordinates to world space after zoom
        let (new_world_x, new_world_y) = self.screen_to_world(x, y);

        // Adjust camera position to keep cursor point stable
        self.camera.position.x += world_x - new_world_x;
        self.camera.position.y += world_y - new_world_y;

        self.render();
    }

    pub fn handle_mouse_leave(&mut self) {
        if self.hover_effect.is_visible() {
            self.hover_effect.hide(&mut self.world);
            self.render();
        }
    }

    /// Requests a render to occur during the next tick.
    pub fn render(&mut self) {
        self.needs_render = true;
    }

    /// Unconditionally called every 16 ms, returns "true" if the framebuffer
    /// was refreshed.
    pub fn tick(&mut self) -> bool {
        if !self.needs_render {
            return false;
        }

        let width = 5.0 * self.camera.width / self.window_size.0 as f64;
        self.hover_effect
            .update_stroke_width(width, &mut self.world, self.renderer.gl());

        self.renderer.render(&mut self.world, &self.camera);
        self.renderer.check_gl_error("Scene render");
        self.needs_render = false;
        true // Frame was rendered
    }

    pub fn resize(&mut self, physical_width: u32, physical_height: u32) {
        self.window_size = (physical_width, physical_height);
        self.renderer.set_viewport(Viewport {
            left: 0.0,
            top: 0.0,
            width: physical_width as f64,
            height: physical_height as f64,
        });
        let window_aspect = physical_width as f64 / physical_height as f64;
        self.camera.height = self.camera.width / window_aspect;

        self.renderer.render(&mut self.world, &self.camera);
        self.renderer.check_gl_error("Scene render");
    }

    pub fn destroy(&mut self) {
        let mut geo_query = self.world.query::<&mut Geometry>();
        for mut geo in geo_query.iter_mut(&mut self.world) {
            geo.destroy(self.renderer.gl());
        }

        let mut mat_query = self.world.query::<&mut Material>();
        for mut mat in mat_query.iter_mut(&mut self.world) {
            mat.destroy(self.renderer.gl());
        }

        // TODO: despawn the entities too
    }

    pub fn project(&self) -> Option<&Project> {
        self.project.as_ref()
    }

    pub fn project_mut(&mut self) -> Option<&mut Project> {
        self.project.as_mut()
    }

    pub fn apply_theme(&mut self, theme: Theme) {
        let mut project = self.project.take().unwrap();
        let mut material = self.world.get_mut::<Material>(self.layer_material).unwrap();
        match theme {
            Theme::Light => {
                material.set_blending(BlendMode::Additive);
                project.apply_light_scheme();
            }
            Theme::Dark => {
                material.set_blending(BlendMode::SourceOver);
                project.apply_rainbow_scheme();
            }
        }
        for layer in project.layers() {
            let id = layer.mesh.unwrap();
            let mesh = self.world.get_mut::<Mesh>(id).unwrap().into_inner();
            mesh.set_vec4("color", layer.color);
        }
        self.project = Some(project);
        self.render();
    }

    fn screen_to_world(&self, screen_x: u32, screen_y: u32) -> (f64, f64) {
        let ndc_x = (screen_x as f64 / self.window_size.0 as f64) * 2.0 - 1.0;
        let ndc_y = -((screen_y as f64 / self.window_size.1 as f64) * 2.0 - 1.0);
        let world = self.camera.unproject(Point3::new(ndc_x, ndc_y, 0.0));
        (world.x, world.y)
    }
}

impl Drop for AppController {
    fn drop(&mut self) {
        self.destroy();
    }
}
