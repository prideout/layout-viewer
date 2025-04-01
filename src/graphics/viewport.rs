#![allow(dead_code)]

#[derive(Debug, Clone, Copy)]
pub struct Viewport {
    pub left: f64,
    pub top: f64,
    pub width: f64,
    pub height: f64,
}

impl Viewport {
    pub fn new(left: f64, top: f64, width: f64, height: f64) -> Self {
        Self {
            left,
            top,
            width,
            height,
        }
    }

    pub fn aspect_ratio(&self) -> f64 {
        self.width / self.height
    }
}
