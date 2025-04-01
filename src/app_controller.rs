use crate::app_shaders::FRAGMENT_SHADER;
use crate::app_shaders::VERTEX_SHADER;
use crate::core::Layer;
use crate::graphics::BlendMode;
use crate::graphics::Camera;
use crate::graphics::Geometry;
use crate::graphics::Material;
use crate::graphics::MaterialId;
use crate::graphics::Mesh;
use crate::graphics::Renderer;
use crate::graphics::Scene;
use crate::graphics::Viewport;
use crate::hover_effect::HoverEffect;
use crate::hover_effect::HoverParams;
use crate::Project;

use geo::TriangulateEarcut;

type Point3 = nalgebra::Point3<f64>;

/// Encapsulates high-level application logic common to all platforms.
pub struct AppController {
    window_size: (u32, u32),
    renderer: Renderer,
    camera: Camera,
    scene: Scene,
    is_dragging: bool,
    last_mouse_pos: Option<(u32, u32)>,
    zoom_speed: f64,
    needs_render: bool,
    project: Option<Project>,
    hover_effect: HoverEffect,
    layer_material: MaterialId,
}

pub enum Theme {
    Light,
    Dark,
}

impl AppController {
    pub fn new(
        renderer: Renderer,
        mut scene: Scene,
        physical_width: u32,
        physical_height: u32,
    ) -> Self {
        let camera = Camera::new(Point3::new(0.0, 0.0, 0.0), 128.0, 128.0, -1.0, 1.0);

        let hover_effect = HoverEffect::new(&mut scene);

        let layer_material = Material::new(VERTEX_SHADER, FRAGMENT_SHADER);
        let layer_material_id = scene.add_material(layer_material);

        Self {
            window_size: (physical_width, physical_height),
            renderer,
            camera,
            scene,
            is_dragging: false,
            last_mouse_pos: None,
            zoom_speed: 0.05,
            needs_render: true,
            project: None,
            hover_effect,
            layer_material: layer_material_id,
        }
    }

    pub fn get_mesh_for_layer_mut(&mut self, layer_index: usize) -> &mut Mesh {
        let layer = &self.project.as_ref().unwrap().layers()[layer_index];
        let id = layer.mesh.unwrap();
        self.scene.get_mesh_mut(&id).unwrap()
    }

    pub fn set_project(&mut self, mut project: Project) {
        let mut alpha = 0.6; // looks ok for 4004 & 6502
        if project.layers().len() > 10 {
            alpha = 0.05;
        }
        for layer in project.layers_mut() {
            layer.color.w = alpha;
        }

        self.create_meshes(project.layers_mut());

        self.hover_effect.move_to_back(&mut self.scene);

        let bounds = project.bounds();
        self.camera.fit_to_bounds(self.window_size, bounds);

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
                    scene: &mut self.scene,
                    gl: self.renderer.gl(),
                });
                self.render();
            }
        } else if self.hover_effect.is_visible() {
            self.hover_effect.hide(&mut self.scene);
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
            self.hover_effect.hide(&mut self.scene);
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
        self.hover_effect.update_stroke_width(width, &mut self.scene, self.renderer.gl());

        self.renderer.render(&mut self.scene, &self.camera);
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

        self.renderer.render(&mut self.scene, &self.camera);
        self.renderer.check_gl_error("Scene render");
    }

    pub fn destroy(&mut self) {
        self.scene.destroy(self.renderer.gl());
    }

    pub fn project(&self) -> Option<&Project> {
        self.project.as_ref()
    }

    pub fn project_mut(&mut self) -> Option<&mut Project> {
        self.project.as_mut()
    }

    pub fn apply_theme(&mut self, theme: Theme) {
        let mut project = self.project.take().unwrap();
        let material = self.scene.get_material_mut(&self.layer_material).unwrap();
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
            if let Some(mesh) = layer.mesh {
                let mesh = self.scene.get_mesh_mut(&mesh).unwrap();
                mesh.set_vec4("color", layer.color);
            }
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

    fn create_meshes(&mut self, layers: &mut [Layer]) {
        for layer in layers {
            let geometry = create_layer_geometry(layer);
            let geometry_id = self.scene.add_geometry(geometry);
            let mesh = Mesh::new(geometry_id, self.layer_material);
            let id = self.scene.add_mesh(mesh);
            layer.mesh = Some(id);
        }
    }
}

impl Drop for AppController {
    fn drop(&mut self) {
        self.destroy();
    }
}

/// Triangulates polygons and appends them to a vertex buffer.
fn create_layer_geometry(layer: &Layer) -> Geometry {
    let mut geometry = Geometry::new();

    // Process each polygon in the layer
    // This is the slowest part of the load process (at the time of this writing)
    for element_instance in &layer.element_instances {
        let triangles = element_instance.polygon.earcut_triangles_raw();

        let vertex_offset = geometry.positions.len() as u32 / 3;

        for coord in triangles.vertices.chunks(2) {
            let x = coord[0];
            let y = coord[1];
            geometry.positions.push(x as f32);
            geometry.positions.push(y as f32);
            geometry.positions.push(0.0);
        }

        geometry.indices.extend(
            triangles
                .triangle_indices
                .iter()
                .map(|i| (*i as u32 + vertex_offset)),
        );
    }

    geometry
}
