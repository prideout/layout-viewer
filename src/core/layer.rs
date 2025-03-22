use crate::graphics::BoundingBox;
use gds21::GdsBoundary;
use gds21::GdsPath;
use gds21::GdsPoint;
use geo::AffineOps;
use geo::AffineTransform;
use geo::BoundingRect;
use geo::LineString;
use geo::Rect;
use i_overlay::mesh::stroke::offset::StrokeOffset;
use i_overlay::mesh::style::LineCap;
use i_overlay::mesh::style::LineJoin;
use i_overlay::mesh::style::StrokeStyle;
use nalgebra::Vector4;

type Polygon = geo::Polygon<f64>;
type Vec2d = geo::Point<f64>;

pub struct Layer {
    index: i16,
    pub polygons: Vec<Polygon>,
    pub bounds: BoundingBox,
    pub paths: Vec<GdsPath>,
    pub boundaries: Vec<GdsBoundary>,
    pub color: Vector4<f32>, // RGBA color for this layer
    pub visible: bool,
}

impl Layer {
    pub fn new(index: i16) -> Self {
        Self {
            index,
            polygons: vec![],
            bounds: BoundingBox::new(),
            paths: Vec::new(),
            boundaries: Vec::new(),
            color: Vector4::new(0.0, 0.0, 0.0, 1.0), // Default to black
            visible: true,
        }
    }

    pub fn index(&self) -> i16 {
        self.index
    }

    pub fn update_bounds(&mut self) {
        self.bounds = BoundingBox::new();

        for polygon in &self.polygons {
            if let Some(bbox) = polygon.bounding_rect() {
                let layer_bbox = BoundingBox::from(bbox);
                self.bounds.encompass(&layer_bbox);
            }
        }
    }

    pub fn add_boundary_element(&mut self, boundary: &GdsBoundary, transform: &AffineTransform) {
        let points: Vec<Vec2d> = boundary.xy.iter().map(gds_to_geo_point).collect();
        if points.len() < 3 {
            log::warn!("Boundary has less than 3 points, skipping");
            return;
        }

        let polygon = Polygon::new(LineString::from(points), vec![]);
        let transformed = polygon.affine_transform(transform);
        self.polygons.push(transformed);
    }

    pub fn add_path_element(&mut self, path: &GdsPath, transform: &AffineTransform) {
        if path.xy.len() < 2 {
            return;
        }

        let half_width = path.width.unwrap_or(0) as f64 / 2.0;
        let path_type = path
            .path_type
            .map(PathType::from)
            .unwrap_or(PathType::Standard);

        let outline_points = self.create_path_outline(&path.xy, half_width, path_type);

        // Create and transform the polygon
        let polygon = Polygon::new(LineString::from(outline_points), vec![]);
        let transformed = polygon.affine_transform(transform);
        self.polygons.push(transformed);
    }

    // Private helper functions
    fn create_path_outline(
        &mut self,
        spine_points: &[GdsPoint],
        half_width: f64,
        path_type: PathType,
    ) -> Vec<Vec2d> {
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
}

fn gds_to_geo_point(p: &GdsPoint) -> Vec2d {
    Vec2d::new(p.x as f64, p.y as f64)
}

fn gds_point_to_array(p: &GdsPoint) -> [f64; 2] {
    [p.x as f64, p.y as f64]
}

fn array_to_geo_point(t: &[f64; 2]) -> Vec2d {
    Vec2d::new(t[0], t[1])
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

impl From<Rect<f64>> for BoundingBox {
    fn from(rect: Rect<f64>) -> Self {
        Self {
            min_x: rect.min().x,
            min_y: rect.min().y,
            max_x: rect.max().x,
            max_y: rect.max().y,
        }
    }
}
