use gds21::GdsBoundary;
use gds21::GdsPath;
use gds21::GdsPoint;
use geo::TriangulateEarcut;
use i_overlay::mesh::stroke::offset::StrokeOffset;
use i_overlay::mesh::style::LineCap;
use i_overlay::mesh::style::LineJoin;
use i_overlay::mesh::style::StrokeStyle;

type Polygon = geo::Polygon<f64>;
type LineString = geo::LineString<f64>;
type Vector = geo::Point<f64>;

/// Bundles a 2D polygon definition with some metadata.
/// Think of this as a rubber stamp, not a positioned instance.
pub struct Element {
    pub shape: GdsSourceShape,
    #[allow(dead_code)]
    pub polygon: Polygon,
    #[allow(dead_code)]
    pub triangles: Vec<u32>,
    #[allow(dead_code)]
    pub vertices: Vec<f32>,
}

pub enum GdsSourceShape {
    Boundary(GdsBoundary),
    Path(GdsPath),
}

impl Element {
    pub fn from_boundary(boundary: GdsBoundary) -> Self {
        let points: Vec<Vector> = boundary.xy.iter().map(gds_to_geo_point).collect();
        let polygon = Polygon::new(LineString::from(points), vec![]);
        let earcut_result = polygon.earcut_triangles_raw();
        let mut vertices = Vec::with_capacity(earcut_result.vertices.len());
        for coord in earcut_result.vertices.chunks(2) {
            vertices.push(coord[0] as f32);
            vertices.push(coord[1] as f32);
        }
        let mut triangles = Vec::with_capacity(earcut_result.triangle_indices.len());
        for i in earcut_result.triangle_indices {
            triangles.push(i as u32);
        }
        Self {
            shape: GdsSourceShape::Boundary(boundary),
            polygon,
            triangles,
            vertices,
        }
    }

    pub fn from_path(path: GdsPath) -> Self {
        let half_width = path.width.unwrap_or(0) as f64 / 2.0;

        let path_type = path
            .path_type
            .map(PathType::from)
            .unwrap_or(PathType::Standard);

        let outline_points = create_path_outline(&path.xy, half_width, path_type);
        let polygon = Polygon::new(LineString::from(outline_points), vec![]);
        let earcut_result = polygon.earcut_triangles_raw();
        let mut vertices = Vec::with_capacity(earcut_result.vertices.len());
        for coord in earcut_result.vertices.chunks(2) {
            vertices.push(coord[0] as f32);
            vertices.push(coord[1] as f32);
        }
        let mut triangles = Vec::with_capacity(earcut_result.triangle_indices.len());
        for i in earcut_result.triangle_indices {
            triangles.push(i as u32);
        }

        Self {
            shape: GdsSourceShape::Path(path.clone()),
            polygon,
            triangles,
            vertices,
        }
    }
}

fn gds_to_geo_point(p: &GdsPoint) -> Vector {
    Vector::new(p.x as f64, p.y as f64)
}

fn gds_point_to_array(p: &GdsPoint) -> [f64; 2] {
    [p.x as f64, p.y as f64]
}

fn array_to_geo_point(t: &[f64; 2]) -> Vector {
    Vector::new(t[0], t[1])
}

fn create_path_outline(
    spine_points: &[GdsPoint],
    half_width: f64,
    path_type: PathType,
) -> Vec<Vector> {
    let start_cap = match path_type {
        PathType::Round => LineCap::Round(0.1),
        PathType::Extended => LineCap::Square,
        PathType::Standard => LineCap::Butt,
    };

    let end_cap = match path_type {
        PathType::Round => LineCap::Round(0.1),
        PathType::Extended => LineCap::Square,
        PathType::Standard => LineCap::Butt,
    };

    let style = StrokeStyle::new(half_width * 2.0)
        .line_join(LineJoin::Miter(1.0))
        .start_cap(start_cap)
        .end_cap(end_cap);

    let spine_points: Vec<[f64; 2]> = spine_points.iter().map(gds_point_to_array).collect();
    let shapes: Vec<Vec<Vec<[f64; 2]>>> = spine_points.stroke(style, false);

    if let Some(first_shape) = shapes.first() {
        if let Some(first_contour) = first_shape.first() {
            return first_contour.iter().map(array_to_geo_point).collect();
        }
    }

    log::warn!("Empty contour for path.");
    vec![]
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum PathType {
    Standard = 0,
    Round = 1,
    Extended = 2,
}

impl From<i16> for PathType {
    fn from(value: i16) -> Self {
        match value {
            1 => PathType::Round,
            2 => PathType::Extended,
            _ => PathType::Standard,
        }
    }
}
