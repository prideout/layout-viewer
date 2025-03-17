use crate::{gl_camera::Camera, gl_renderer::Renderer, gl_viewport::Viewport, Scene};
use nalgebra::Point3;

pub struct Controller {
    renderer: Renderer,
    camera: Camera,
    scene: Scene,
    is_dragging: bool,
    last_mouse_pos: Option<(f32, f32)>,
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
        }
    }

    pub fn handle_mouse_press(&mut self, x: f32, y: f32) {
        self.is_dragging = true;
        self.last_mouse_pos = Some((x, y));
    }

    pub fn handle_mouse_release(&mut self) {
        self.is_dragging = false;
        self.last_mouse_pos = None;
    }

    pub fn handle_mouse_move(&mut self, x: f32, y: f32) {
        if self.is_dragging {
            if let Some((last_x, last_y)) = self.last_mouse_pos {
                // Convert pixel movement to world space movement
                let viewport = self.renderer.get_viewport();
                let dx = (x - last_x) * self.camera.width / viewport.width;
                let dy = (y - last_y) * self.camera.height / viewport.height;

                // Move camera in the opposite direction of mouse movement
                let mut pos = self.camera.position;
                pos.x -= dx;
                pos.y += dy; // Invert Y since screen coordinates are top-down
                self.camera.position = pos;
            }
            self.last_mouse_pos = Some((x, y));
        }
    }

    pub fn render(&mut self) {
        self.renderer.render(&mut self.scene, &self.camera);
        self.renderer.check_gl_error("Scene render");
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
