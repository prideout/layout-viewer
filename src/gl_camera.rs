type Mat4 = nalgebra::Matrix4<f32>;
type Point = nalgebra::Point3<f32>;

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
        Mat4::new_translation(&(-self.position.coords))
    }

    /// Sets the world space width and height of the near projection quad.
    pub fn set_size(&mut self, width: f32, height: f32) {
        self.width = width;
        self.height = height;
    }
}
