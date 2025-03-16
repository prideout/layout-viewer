#![allow(dead_code)]

use gds21::{GdsBoundary, GdsPath};
use geo::{AffineOps, AffineTransform, BoundingRect, LineString};
use i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_overlay::mesh::stroke::offset::StrokeOffset;
use i_overlay::mesh::style::{LineCap, LineJoin, StrokeStyle};
use std::collections::HashMap;

use crate::bounds::BoundingBox;
use crate::cells::CellId;
use crate::vec2d::Vec2d;

type Polygon = geo::Polygon<f64>;

const ARC_SUBDIVISIONS: usize = 8;

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

pub struct RenderLayer {
    pub polygons: HashMap<CellId, Vec<Polygon>>,
    pub bounds: BoundingBox,
}

impl RenderLayer {
    pub fn new() -> Self {
        Self {
            polygons: HashMap::new(),
            bounds: BoundingBox::new(),
        }
    }

    pub fn update_bounds(&mut self) {
        self.bounds = BoundingBox::new();

        for cell_polygons in self.polygons.values() {
            for polygon in cell_polygons {
                if let Some(bbox) = polygon.bounding_rect() {
                    let layer_bbox = BoundingBox::from(bbox);
                    self.bounds.encompass(&layer_bbox);
                }
            }
        }
    }

    pub fn add_boundary_element(
        &mut self,
        cell_id: CellId,
        boundary: &GdsBoundary,
        transform: &AffineTransform,
    ) {
        let points: Vec<Vec2d> = boundary.xy.iter().map(Vec2d::from).collect();

        // GDS requires the last point to equal the first point
        if points.len() >= 3 {
            let polygon = Polygon::new(LineString::from(points), vec![]);
            let transformed = polygon.affine_transform(transform);
            self.polygons.entry(cell_id).or_default().push(transformed);
        }
    }

    pub fn add_path_element(
        &mut self,
        cell_id: CellId,
        path: &GdsPath,
        transform: &AffineTransform,
    ) {
        if path.xy.len() < 2 {
            return;
        }

        let half_width = path.width.unwrap_or(0) as f64 / 2.0;
        let points: Vec<Vec2d> = path.xy.iter().map(Vec2d::from).collect();
        let path_type = path
            .path_type
            .map(PathType::from)
            .unwrap_or(PathType::Standard);

        let outline_points = self.create_path_outline(&points, half_width, path_type);

        // Create and transform the polygon
        let polygon = Polygon::new(LineString::from(outline_points), vec![]);
        let transformed = polygon.affine_transform(transform);
        self.polygons.entry(cell_id).or_default().push(transformed);
    }

    pub fn get_polygons(&self, cell_id: CellId) -> Option<&[Polygon]> {
        self.polygons.get(&cell_id).map(|v| v.as_slice())
    }

    // Private helper functions
    fn create_path_outline(
        &mut self,
        spine_points: &[Vec2d],
        half_width: f64,
        path_type: PathType,
    ) -> Vec<Vec2d> {
        // At the time of this writing, LineCap is neither
        // copyable nor cloneable.

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

        let shapes: Vec<Vec<Vec<Vec2d>>> = spine_points.stroke(style, false);

        if let Some(first_shape) = shapes.first() {
            if let Some(first_contour) = first_shape.first() {
                return first_contour.clone();
            }
        }

        eprintln!("Empty contour for path.");

        vec![]
    }
}

impl Default for RenderLayer {
    fn default() -> Self {
        Self::new()
    }
}

impl FloatPointCompatible<f64> for Vec2d {
    fn from_xy(x: f64, y: f64) -> Self {
        Self(x, y)
    }

    fn x(&self) -> f64 {
        self.0
    }

    fn y(&self) -> f64 {
        self.1
    }
}
