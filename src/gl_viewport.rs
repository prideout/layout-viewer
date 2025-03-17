#[derive(Debug, Clone, Copy)]
pub struct Viewport {
    pub left: f32,
    pub top: f32,
    pub width: f32,
    pub height: f32,
}

impl Viewport {
    pub fn new(left: f32, top: f32, width: f32, height: f32) -> Self {
        Self { left, top, width, height }
    }

    pub fn aspect_ratio(&self) -> f32 {
        self.width / self.height
    }
} 