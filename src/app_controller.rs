use crate::graphics::Camera;
use crate::graphics::Geometry;
use crate::graphics::Material;
use crate::graphics::Mesh;
use crate::graphics::Renderer;
use crate::graphics::Scene;
use crate::graphics::Viewport;
use crate::core::Layer;
use crate::core::PickResult;
use crate::app_shaders::FRAGMENT_SHADER;
use crate::app_shaders::VERTEX_SHADER;
use crate::Project;
use geo::TriangulateEarcut;
use nalgebra::Point3;

/// Encapsulates high-level application logic common to all platforms.
pub struct AppController {
    window_size: (u32, u32),
    renderer: Renderer,
    camera: Camera,
    scene: Scene,
    is_dragging: bool,
    last_mouse_pos: Option<(f32, f32)>,
    zoom_speed: f32,
    needs_render: bool,
    project: Option<Project>,
    hovered_cell: Option<PickResult>,
}

impl AppController {
    pub fn new(
        renderer: Renderer,
        scene: Scene,
        physical_width: u32,
        physical_height: u32,
    ) -> Self {
        let camera = Camera::new(Point3::new(0.0, 0.0, 0.0), 128.0, 128.0, -1.0, 1.0);
        Self {
            window_size: (physical_width, physical_height),
            renderer,
            camera,
            scene,
            is_dragging: false,
            last_mouse_pos: None,
            zoom_speed: 0.05,
            needs_render: true, // Initial render needed
            project: None,
            hovered_cell: None,
        }
    }

    pub fn set_project(&mut self, mut project: Project) {
        let stats = project.stats();
        log::info!("Number of structs: {}", stats.struct_count);
        log::info!("Number of polygons: {}", stats.polygon_count);
        log::info!("Number of paths: {}", stats.path_count);
        log::info!(
            "Number of layers: {}",
            (project.highest_layer() + 1) as usize
        );

        let mut alpha = 0.6; // looks ok for 4004 & 6502
        if project.layers().len() > 10 {
            alpha = 0.05;
        }
        for layer in project.layers_mut() {
            layer.color.w = alpha;
        }

        populate_scene(project.layers(), &mut self.scene);

        let bounds = project.bounds();
        self.camera.fit_to_bounds(self.window_size, bounds);

        self.project = Some(project);

        self.render();
    }

    pub fn handle_mouse_press(&mut self, x: u32, y: u32) {
        self.is_dragging = true;
        self.last_mouse_pos = Some((x as f32, y as f32));
    }

    pub fn handle_mouse_release(&mut self) {
        self.is_dragging = false;
        self.last_mouse_pos = None;
    }

    pub fn handle_mouse_move(&mut self, x: u32, y: u32) {
        if self.is_dragging {
            if let Some((last_x, last_y)) = self.last_mouse_pos {
                // Convert pixel movement to world space movement
                let viewport = self.renderer.get_viewport();
                let dx = (x as f32 - last_x) * self.camera.width / viewport.width;
                let dy = (y as f32 - last_y) * self.camera.height / viewport.height;

                // Move camera in the opposite direction of mouse movement
                let mut pos = self.camera.position;
                pos.x -= dx;
                pos.y += dy; // Invert Y since screen coordinates are top-down
                self.camera.position = pos;
            }
            self.last_mouse_pos = Some((x as f32, y as f32));
        }

        // Convert screen coordinates to world space
        let (world_x, world_y) = self.screen_to_world(x, y);
        if let Some(project) = self.project() {
            if let Some(result) = project.pick_cell(world_x, world_y) {
                if self.hovered_cell != Some(result.clone()) {
                    log::info!("Picked {:?}", &result);
                    self.hovered_cell = Some(result);
                }
            } else if self.hovered_cell.is_some() {
                self.hovered_cell = None;
            }
        }
    }

    pub fn handle_mouse_wheel(&mut self, x: u32, y: u32, delta: f32) {
        // Ignore very small deltas that might be touchpad bounce
        const MIN_DELTA: f32 = 0.01;
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
        self.camera.position.x += (world_x - new_world_x) as f32;
        self.camera.position.y += (world_y - new_world_y) as f32;
    }

    fn screen_to_world(&self, screen_x: u32, screen_y: u32) -> (f64, f64) {
        let viewport = self.renderer.get_viewport();

        // Convert screen coordinates to normalized device coordinates (-1 to 1)
        let ndc_x = (screen_x as f64 / viewport.width as f64) * 2.0 - 1.0;
        let ndc_y = -((screen_y as f64 / viewport.height as f64) * 2.0 - 1.0); // Flip Y axis

        // Convert to world space
        let world_x = self.camera.position.x as f64 + ndc_x * self.camera.width as f64 / 2.0;
        let world_y = self.camera.position.y as f64 + ndc_y * self.camera.height as f64 / 2.0;

        (world_x, world_y)
    }

    pub fn render(&mut self) {
        self.needs_render = true;
    }

    pub fn tick(&mut self) -> bool {
        if !self.needs_render {
            return false;
        }
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
            width: physical_width as f32,
            height: physical_height as f32,
        });
        if let Some(project) = self.project() {
            let bounds = project.bounds();
            self.camera.fit_to_bounds(self.window_size, bounds);
        }
    }

    pub fn destroy(&mut self) {
        self.scene.destroy(self.renderer.gl());
    }

    pub fn scene(&mut self) -> &mut Scene {
        &mut self.scene
    }

    pub fn project(&self) -> Option<&Project> {
        self.project.as_ref()
    }

    pub fn project_mut(&mut self) -> Option<&mut Project> {
        self.project.as_mut()
    }

    pub fn camera(&self) -> &Camera {
        &self.camera
    }
}

impl Drop for AppController {
    fn drop(&mut self) {
        self.destroy();
    }
}

pub fn populate_scene(layers: &[Layer], scene: &mut Scene) {
    let mut material = Material::new(VERTEX_SHADER, FRAGMENT_SHADER);

    material.set_blending(true);

    let material_id = scene.add_material(material);

    for layer in layers {
        let geometry = create_layer_geometry(layer);
        let geometry_id = scene.add_geometry(geometry);
        let mut mesh = Mesh::new(geometry_id, material_id);

        // Set the color uniform using the layer's color
        mesh.set_vec4("color", layer.color);

        scene.add_mesh(mesh);
    }
}

/// Triangulates polygons and appends them to a vertex buffer.
fn create_layer_geometry(layer: &Layer) -> Geometry {
    let mut geometry = Geometry::new();

    // Process each polygon in the layer
    for polygon in &layer.polygons {
        let triangles = polygon.earcut_triangles_raw();

        let vertex_offset = geometry.positions.len() as u32 / 3;

        for coord in triangles.vertices.chunks(2) {
            let x = coord[0];
            let y = coord[1];
            geometry
                .positions
                .extend_from_slice(&[x as f32, y as f32, 0.0]);
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
