#![allow(dead_code)]

use gds21::{GdsBoundary, GdsPath};
use geo::{AffineOps, AffineTransform, BoundingRect, LineString};
use std::collections::HashMap;
use std::f64::consts::PI;

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
        points: &[Vec2d],
        half_width: f64,
        path_type: PathType,
    ) -> Vec<Vec2d> {
        let mut outline_points = Vec::with_capacity(points.len() * 2 + 1);

        // Add start cap
        if let Some((dir, norm)) = self.get_segment_vectors(&points[0], &points[1]) {
            self.add_cap(
                &mut outline_points,
                points[0],
                dir,
                norm.scale(half_width),
                path_type,
                true,
            );
        }

        // Add left path side with miter joints
        self.add_left_path_side(&mut outline_points, points, half_width);

        // Add end cap
        let last_idx = points.len() - 1;
        if let Some((dir, norm)) =
            self.get_segment_vectors(&points[last_idx - 1], &points[last_idx])
        {
            self.add_cap(
                &mut outline_points,
                points[last_idx],
                dir,
                norm.scale(half_width),
                path_type,
                false,
            );
        }

        // Add right path
        self.add_right_path_side(&mut outline_points, points, half_width);

        // Close the polygon
        if let Some(first) = outline_points.first().copied() {
            outline_points.push(first);
        }

        outline_points
    }

    fn get_segment_vectors(&self, p1: &Vec2d, p2: &Vec2d) -> Option<(Vec2d, Vec2d)> {
        let diff = p2.subtract(p1);
        let len = diff.length();
        if len == 0.0 {
            return None;
        }
        let dir = diff.scale(1.0 / len);
        let norm = Vec2d(-dir.y(), dir.x());
        Some((dir, norm))
    }

    fn add_cap(
        &mut self,
        points: &mut Vec<Vec2d>,
        center: Vec2d,
        dir: Vec2d,
        norm: Vec2d,
        path_type: PathType,
        is_start: bool,
    ) {
        let dir = if is_start { dir } else { dir.flip() };

        match path_type {
            PathType::Round => {
                self.add_round_cap(points, center, dir, norm.length());
            }
            PathType::Extended => {
                self.add_extended_cap(points, center, dir, norm.length());
            }
            PathType::Standard => {
                // Square end
                points.push(center.subtract(&norm));
            }
        }
    }

    fn add_left_path_side(
        &mut self,
        outline_points: &mut Vec<Vec2d>,
        points: &[Vec2d],
        half_width: f64,
    ) {
        for i in 0..points.len() - 1 {
            if i < points.len() - 2 {
                let segment = [points[i], points[i + 1], points[i + 2]];
                self.add_miter_joint(outline_points, &segment, half_width, true);
            } else if let Some((_, norm)) = self.get_segment_vectors(&points[i], &points[i + 1]) {
                let scaled_norm = norm.scale(half_width);
                outline_points.push(points[i + 1].add(&scaled_norm));
            }
        }
    }

    fn add_right_path_side(
        &mut self,
        outline_points: &mut Vec<Vec2d>,
        points: &[Vec2d],
        half_width: f64,
    ) {
        for i in (0..points.len() - 2).rev() {
            if i < points.len() - 2 {
                let segment = [points[i], points[i + 1], points[i + 2]];
                self.add_miter_joint(outline_points, &segment, half_width, false);
            } else if let Some((_, norm)) = self.get_segment_vectors(&points[i], &points[i + 1]) {
                let scaled_norm = norm.scale(half_width);
                outline_points.push(points[i + 1].subtract(&scaled_norm));
            }
        }
    }

    fn add_miter_joint(
        &mut self,
        outline_points: &mut Vec<Vec2d>,
        segment_points: &[Vec2d; 3],
        half_width: f64,
        is_left_side: bool,
    ) {
        let [p1, p2, p3] = segment_points;

        if let (Some((_, norm1)), Some((_, norm2))) = (
            self.get_segment_vectors(p1, p2),
            self.get_segment_vectors(p2, p3),
        ) {
            let (norm1, norm2) = if is_left_side {
                (norm1, norm2)
            } else {
                (norm1.flip(), norm2.flip())
            };

            let scaled_norm1 = norm1.scale(half_width);
            let scaled_norm2 = norm2.scale(half_width);

            // Calculate offset lines' start points
            let offset1 = p1.add(&scaled_norm1);
            let offset2 = p2.add(&scaled_norm2);

            // Direction vectors for the two lines
            let dir1 = p2.subtract(p1);
            let offset_diff = offset2.subtract(&offset1);

            // Calculate intersection parameter using cross products
            let t = offset_diff.cross(&dir1) / norm1.cross(&dir1);

            // Find intersection point by interpolating along the first offset line
            let miter = offset1.lerp(&p2.add(&scaled_norm1), t);
            outline_points.push(miter);
        }
    }

    fn add_round_cap(
        &mut self,
        points: &mut Vec<Vec2d>,
        center: Vec2d,
        dir: Vec2d,
        half_width: f64,
    ) {
        for i in 0..=ARC_SUBDIVISIONS {
            let angle = PI * (i as f64) / (ARC_SUBDIVISIONS as f64);
            let nx = -dir.y() * angle.cos() + dir.x() * angle.sin();
            let ny = dir.x() * angle.cos() + dir.y() * angle.sin();
            let offset = Vec2d::new(nx, ny).scale(half_width);
            points.push(center.add(&offset));
        }
    }

    fn add_extended_cap(
        &mut self,
        points: &mut Vec<Vec2d>,
        center: Vec2d,
        dir: Vec2d,
        half_width: f64,
    ) {
        let ext = half_width;
        let normal = Vec2d(-dir.y(), dir.x());
        let scaled_normal = normal.scale(half_width);
        let scaled_dir = dir.scale(ext);

        points.push(center.add(&scaled_normal).add(&scaled_dir));
        points.push(center.subtract(&scaled_normal).add(&scaled_dir));
    }
}

impl Default for RenderLayer {
    fn default() -> Self {
        Self::new()
    }
}
