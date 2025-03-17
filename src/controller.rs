use crate::{gl_camera::Camera, gl_renderer::Renderer, gl_viewport::Viewport, Scene};
use nalgebra::Point3;

pub struct Controller {
    renderer: Renderer,
    camera: Camera,
    scene: Scene,
    is_dragging: bool,
    last_mouse_pos: Option<(f32, f32)>,
    zoom_speed: f32,
    needs_render: bool,
}

impl Controller {
    pub fn new(
        renderer: Renderer,
        scene: Scene,
        physical_width: u32,
        physical_height: u32,
    ) -> Self {
        let (width, height) = calculate_normalized_dimensions(physical_width, physical_height);
        let camera = Camera::new(Point3::new(0.0, 0.0, 0.0), width, height, -1.0, 1.0);
        Self {
            renderer,
            camera,
            scene,
            is_dragging: false,
            last_mouse_pos: None,
            zoom_speed: 0.05,
            needs_render: true, // Initial render needed
        }
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
        self.camera.position.x += world_x - new_world_x;
        self.camera.position.y += world_y - new_world_y;
    }

    fn screen_to_world(&self, screen_x: u32, screen_y: u32) -> (f32, f32) {
        let viewport = self.renderer.get_viewport();

        // Convert screen coordinates to normalized device coordinates (-1 to 1)
        let ndc_x = (screen_x as f32 / viewport.width) * 2.0 - 1.0;
        let ndc_y = -((screen_y as f32 / viewport.height) * 2.0 - 1.0); // Flip Y axis

        // Convert to world space
        let world_x = self.camera.position.x + ndc_x * self.camera.width / 2.0;
        let world_y = self.camera.position.y + ndc_y * self.camera.height / 2.0;

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
        self.renderer.set_viewport(Viewport {
            left: 0.0,
            top: 0.0,
            width: physical_width as f32,
            height: physical_height as f32,
        });
        let (width, height) = calculate_normalized_dimensions(physical_width, physical_height);
        self.camera.set_size(width, height);
    }

    pub fn cleanup(&mut self) {
        self.scene.destroy(self.renderer.gl());
    }
}

fn calculate_normalized_dimensions(width: u32, height: u32) -> (f32, f32) {
    let aspect_ratio = width as f32 / height as f32;
    if aspect_ratio > 1.0 {
        (aspect_ratio * 2.0, 2.0) // height is -1 to +1, width scaled by aspect
    } else {
        (2.0, 2.0 / aspect_ratio) // width is -1 to +1, height scaled by aspect
    }
}
