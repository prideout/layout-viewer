use crate::{gl_camera::Camera, gl_renderer::Renderer, Scene};

pub struct Controller {
    renderer: Renderer,
    is_dragging: bool,
    last_mouse_pos: Option<(f32, f32)>,
}

impl Controller {
    pub fn new(renderer: Renderer) -> Self {
        Self {
            renderer,
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

    pub fn handle_mouse_move(&mut self, x: f32, y: f32, camera: &mut Camera) {
        if self.is_dragging {
            if let Some((last_x, last_y)) = self.last_mouse_pos {
                // Convert pixel movement to world space movement
                let viewport = self.renderer.get_viewport();
                let dx = (x - last_x) * camera.width / viewport.width;
                let dy = (y - last_y) * camera.height / viewport.height;

                // Move camera in the opposite direction of mouse movement
                let mut pos = camera.position;
                pos.x -= dx;
                pos.y += dy; // Invert Y since screen coordinates are top-down
                camera.position = pos;
            }
            self.last_mouse_pos = Some((x, y));
        }
    }

    pub fn render(&mut self, scene: &mut Scene, camera: &Camera) {
        self.renderer.render(scene, camera);
    }

    pub fn check_gl_error(&self, location: &str) {
        self.renderer.check_gl_error(location);
    }

    pub fn set_viewport(&mut self, viewport: crate::gl_viewport::Viewport) {
        self.renderer.set_viewport(viewport);
    }

    pub fn gl(&self) -> &glow::Context {
        self.renderer.gl()
    }
}
