use crate::graphics::vectors::*;

use std::fmt;

use crate::graphics::bounds::BoundingBox;

impl fmt::Debug for Camera {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Camera")
            .field("position", &self.position)
            .field("up", &self.up)
            .field("gaze", &self.gaze)
            .field("width", &self.width)
            .field("height", &self.height)
            .field("near", &self.near)
            .field("far", &self.far)
            .finish()
    }
}

pub struct Camera {
    pub position: Point3d,
    pub up: Vector3d,
    pub gaze: Vector3d,
    pub width: f64,
    pub height: f64,
    pub near: f64,
    pub far: f64,
}

impl Camera {
    pub fn new(position: Point3d, width: f64, height: f64, near: f64, far: f64) -> Self {
        Self {
            position,
            up: Vector3d::new(-1.0, 0.0, 0.0),
            gaze: Vector3d::new(0.0, 0.0, -1.0),
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
        let target = self.position + self.gaze;
        Mat4::look_at_rh(&self.position, &target, &self.up)
    }

    /// Projects a world space point to NDC space
    pub fn project(&self, point: Point3d) -> Point3d {
        let view_matrix = self.get_view_matrix();
        let proj_matrix = self.get_projection_matrix();
        let combined = proj_matrix * view_matrix;
        let clip_space = combined * nalgebra::Vector4::new(point.x, point.y, point.z, 1.0);
        let ndc = clip_space / clip_space.w;
        Point3d::new(ndc.x, ndc.y, ndc.z)
    }

    /// Transforms a point from NDC space to world space coordinates
    pub fn unproject(&self, point: Point3d) -> Point3d {
        let view_matrix = self.get_view_matrix();
        let proj_matrix = self.get_projection_matrix();
        let combined = (proj_matrix * view_matrix).try_inverse().unwrap();
        let ndc = nalgebra::Vector4::new(point.x, point.y, point.z, 1.0);
        let world = combined * ndc;
        Point3d::new(world.x, world.y, world.z)
    }

    /// Sets the world space width and height of the near projection quad.
    pub fn set_size(&mut self, width: f64, height: f64) {
        self.width = width;
        self.height = height;
    }

    /// Fits the camera so that it frames the given world-space rectangle.
    pub fn fit_to_bounds(&mut self, window_size: (u32, u32), world_bounds: BoundingBox) {
        let (window_width, window_height) = window_size;
        let window_aspect = window_width as f64 / window_height as f64;

        let world_width = world_bounds.width();
        let world_height = world_bounds.height();
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
        self.position.x = world_bounds.min_x + world_width / 2.0;
        self.position.y = world_bounds.min_y + world_height / 2.0;
    }
}
