use geo::AffineOps;
use geo::AffineTransform;
use geo::TriangulateEarcut;

use crate::graphics::geometry::Geometry;
use crate::graphics::vectors::*;

pub struct Triangulation {
    pub indices: Vec<u32>,
    pub vertices: Vec<Point2f>,
}

impl Triangulation {
    pub fn empty() -> Self {
        Self {
            indices: vec![],
            vertices: vec![],
        }
    }

    pub fn from_polygon(polygon: &Polygon) -> Self {
        let earcut_result = polygon.earcut_triangles_raw();
        let mut vertices = Vec::with_capacity(earcut_result.vertices.len() / 2);
        for coord in earcut_result.vertices.chunks(2) {
            vertices.push(Point2f::new(coord[0] as f32, coord[1] as f32));
        }
        let mut indices = Vec::with_capacity(earcut_result.triangle_indices.len());
        for i in earcut_result.triangle_indices {
            indices.push(i as u32);
        }
        Self { indices, vertices }
    }

    // TODO: Make this more streamlined by taking an f32 AffineTransform and avoiding
    // the back-and-forth conversion to geo::Point.
    pub fn affine_transform(&self, transform: &AffineTransform) -> Self {
        let indices = self.indices.clone();
        let vertices = self
            .vertices
            .iter()
            .map(|v| from_geo(to_geo(v).affine_transform(transform)))
            .collect();
        Self { indices, vertices }
    }

    pub fn append_to(&self, geo: &mut Geometry) {
        let start_index = (geo.positions.len() / 3) as u32;
        for vert in &self.vertices {
            geo.positions.push(vert.x);
            geo.positions.push(vert.y);
            geo.positions.push(0.0);
        }
        for index in &self.indices {
            geo.indices.push(start_index + *index);
        }
    }
}

fn to_geo(p: &Point2f) -> geo::Point<f64> {
    geo::Point::new(p.x as f64, p.y as f64)
}

fn from_geo(p: geo::Point<f64>) -> Point2f {
    Point2f::new(p.x() as f32, p.y() as f32)
}
