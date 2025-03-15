use gds21::GdsPoint;
use geo::Coord;

#[derive(Debug, Clone, Copy)]
pub struct Vec2d(pub(crate) f64, pub(crate) f64);

impl From<&GdsPoint> for Vec2d {
    fn from(p: &GdsPoint) -> Self {
        Vec2d(p.x as f64, p.y as f64)
    }
}

impl From<Vec2d> for Coord<f64> {
    fn from(p: Vec2d) -> Self {
        Coord { x: p.0, y: p.1 }
    }
}

impl Vec2d {
    pub fn new(x: f64, y: f64) -> Self {
        Vec2d(x, y)
    }

    pub fn x(&self) -> f64 {
        self.0
    }
    
    pub fn y(&self) -> f64 {
        self.1
    }
    
    pub fn flip(&self) -> Vec2d {
        Vec2d(-self.0, -self.1)
    }
    
    pub fn scale(&self, factor: f64) -> Vec2d {
        Vec2d(self.0 * factor, self.1 * factor)
    }

    pub fn add(&self, other: &Vec2d) -> Vec2d {
        Vec2d(self.0 + other.0, self.1 + other.1)
    }

    pub fn subtract(&self, other: &Vec2d) -> Vec2d {
        Vec2d(self.0 - other.0, self.1 - other.1)
    }

    pub fn length(&self) -> f64 {
        (self.0 * self.0 + self.1 * self.1).sqrt()
    }

    // Cross product of two 2D vectors (returns scalar)
    pub fn cross(&self, other: &Vec2d) -> f64 {
        self.x() * other.y() - self.y() * other.x()
    }

    // Dot product of two vectors
    pub fn dot(&self, other: &Vec2d) -> f64 {
        self.x() * other.x() + self.y() * other.y()
    }

    // Interpolate between two points
    pub fn lerp(&self, other: &Vec2d, t: f64) -> Vec2d {
        self.add(&other.subtract(self).scale(t))
    }
} 