use gds21::GdsPoint;
use i_overlay::mesh::stroke::offset::StrokeOffset;
use i_overlay::mesh::style::LineCap;
use i_overlay::mesh::style::LineJoin;
use i_overlay::mesh::style::StrokeStyle;

pub type Point = geo::Point<f64>;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PathType {
    Standard = 0,
    Round = 1,
    Extended = 2,
}

pub fn create_path_outline(
    spine_points: &[GdsPoint],
    half_width: f64,
    path_type: PathType,
) -> Vec<Point> {
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

impl From<i16> for PathType {
    fn from(value: i16) -> Self {
        match value {
            1 => PathType::Round,
            2 => PathType::Extended,
            _ => PathType::Standard,
        }
    }
}

fn gds_point_to_array(p: &GdsPoint) -> [f64; 2] {
    [p.x as f64, p.y as f64]
}

fn array_to_geo_point(t: &[f64; 2]) -> Point {
    Point::new(t[0], t[1])
}
