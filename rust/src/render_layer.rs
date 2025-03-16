#![allow(dead_code)]

use gds21::{GdsBoundary, GdsPath, GdsPoint};
use geo::{AffineOps, AffineTransform, BoundingRect, LineString};
use i_overlay::mesh::stroke::offset::StrokeOffset;
use i_overlay::mesh::style::{LineCap, LineJoin, StrokeStyle};
use indexmap::IndexMap;

use crate::bounds::BoundingBox;
use crate::cells::CellId;

type Polygon = geo::Polygon<f64>;
type Vec2d = geo::Point<f64>;

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
    pub polygons: IndexMap<CellId, Vec<Polygon>>,
    pub bounds: BoundingBox,
}

impl RenderLayer {
    pub fn new() -> Self {
        Self {
            polygons: IndexMap::new(),
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
        let points: Vec<Vec2d> = boundary.xy.iter().map(gds_to_geo_point).collect();

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
        let path_type = path
            .path_type
            .map(PathType::from)
            .unwrap_or(PathType::Standard);

        let outline_points = self.create_path_outline(&path.xy, half_width, path_type);

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
        spine_points: &[GdsPoint],
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

        // We cannot add the FloatPointCompatible trait to geo::Point or GdsPoint just use sized arrays.
        let spine_points: Vec<[f64; 2]> = spine_points.iter().map(gds_point_to_array).collect();
        let shapes: Vec<Vec<Vec<[f64; 2]>>> = spine_points.stroke(style, false);

        if let Some(first_shape) = shapes.first() {
            if let Some(first_contour) = first_shape.first() {
                return first_contour.iter().map(array_to_geo_point).collect();
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

fn gds_to_geo_point(p: &GdsPoint) -> Vec2d {
    Vec2d::new(p.x as f64, p.y as f64)
}

fn gds_point_to_array(p: &GdsPoint) -> [f64; 2] {
    [p.x as f64, p.y as f64]
}

fn array_to_geo_point(t: &[f64; 2]) -> Vec2d {
    Vec2d::new(t[0], t[1])
}
