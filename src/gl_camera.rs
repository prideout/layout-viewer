type Mat4 = nalgebra::Matrix4<f32>;
type Point = nalgebra::Point3<f32>;

use std::fmt;

use crate::bounds::BoundingBox;

impl fmt::Debug for Camera {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Camera")
            .field("position", &self.position)
            .field("width", &self.width)
            .field("height", &self.height)
            .field("near", &self.near)
            .field("far", &self.far)
            .finish()
    }
}

pub struct Camera {
    pub position: Point,
    pub width: f32,
    pub height: f32,
    pub near: f32,
    pub far: f32,
}

impl Camera {
    pub fn new(position: Point, width: f32, height: f32, near: f32, far: f32) -> Self {
        Self {
            position,
            width,
            height,
            near,
            far,
        }
    }

    pub fn get_projection_matrix(&self) -> Mat4 {
        let half_width = self.width / 2.0;
        let half_height = self.height / 2.0;

        Mat4::new_orthographic(
            -half_width,  // left
            half_width,   // right
            -half_height, // bottom
            half_height,  // top
            self.near,    // near
            self.far,     // far
        )
    }

    pub fn get_view_matrix(&self) -> Mat4 {
        // For an orthographic camera looking down the Z axis, we need to:
        // 1. Translate by the negative of the camera position
        // 2. Since we're in 2D (looking down Z), we only translate in X and Y
        let mut view = Mat4::identity();
        view[(0, 3)] = -self.position.x;
        view[(1, 3)] = -self.position.y;
        // Z translation is 0 since we're looking down Z axis
        view
    }

    /// Sets the world space width and height of the near projection quad.
    pub fn set_size(&mut self, width: f32, height: f32) {
        self.width = width;
        self.height = height;
    }

    /// Fits the camera so that it frames the given world-space rectangle.
    pub fn fit_to_bounds(&mut self, window_size: (u32, u32), world_bounds: BoundingBox) {
        let (window_width, window_height) = window_size;
        let window_aspect = window_width as f32 / window_height as f32;
        
        let world_width = world_bounds.width() as f32;
        let world_height = world_bounds.height() as f32;
        let world_aspect = world_width / world_height;
        
        if window_aspect > world_aspect {
            // Window is wider than world, so we need to scale based on height
            self.height = world_height;
            self.width = world_height * window_aspect;
        } else {
            // Window is taller than world, so we need to scale based on width
            self.width = world_width;
            self.height = world_width / window_aspect;
        }
        
        // Center the camera on the world bounds
        self.position.x = world_bounds.min_x as f32 + world_width / 2.0;
        self.position.y = world_bounds.min_y as f32 + world_height / 2.0;
    }

}
