use gds21::GdsBoundary;
use gds21::GdsPath;

type Polygon = geo::Polygon<f64>;

/// Bundles a 2D polygon definition with some metadata.
/// Think of this as a rubber stamp, not a positioned instance.
pub struct Element {
    pub shape: GdsSourceShape,
    pub polygon: Polygon,
    pub triangles: Vec<u32>,
    pub vertices: Vec<f32>,
}

pub enum GdsSourceShape {
    Boundary(GdsBoundary),
    Path(GdsPath),
}

impl Element {
    pub fn from_boundary(boundary: GdsBoundary) -> Self {
        Self {
            shape: GdsSourceShape::Boundary(boundary),
            polygon: geo::Polygon::new(geo::LineString(vec![]), vec![]),
            triangles: vec![],
            vertices: vec![],
        }
    }

    pub fn from_path(path: GdsPath) -> Self {
        Self {
            shape: GdsSourceShape::Path(path.clone()),
            polygon: geo::Polygon::new(geo::LineString(vec![]), vec![]),
            triangles: vec![],
            vertices: vec![],
        }
    }
}
