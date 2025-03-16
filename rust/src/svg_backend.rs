use crate::bounds::BoundingBox;
use crate::layer::Layer;
use svg::node::element::{Group, Path};
use svg::Document;

const PRECISION: f64 = 0.0001;
const COLORS: [&str; 10] = [
    "#FF0000", // Red
    "#00FF00", // Green
    "#0000FF", // Blue
    "#FF00FF", // Magenta
    "#00FFFF", // Cyan
    "#FFFF00", // Yellow
    "#FF8000", // Orange
    "#8000FF", // Purple
    "#008080", // Teal
    "#800080", // Dark Purple
];

pub fn generate_svg(layers: &[Layer]) -> String {
    // Get the overall bounding box
    let mut bounds = BoundingBox::new();
    for layer in layers {
        if !layer.bounds.is_empty() {
            bounds.encompass(&layer.bounds);
        }
    }

    // Add padding
    let padding = (bounds.width() + bounds.height()) * 0.05;
    let padded_bounds = bounds.inflate(padding);
    let view_box = (
        padded_bounds.min_y,
        padded_bounds.min_x,
        padded_bounds.height(),
        padded_bounds.width(),
    );

    // Create SVG document
    let mut doc = Document::new()
        .set("viewBox", view_box)
        .set("style", "background-color: #2D2D2D");

    // Add each layer as a group
    for (layer_idx, layer) in layers.iter().enumerate() {
        let color = COLORS[layer_idx % COLORS.len()];
        let mut group = Group::new().set("fill", color).set("opacity", 0.5);

        // Get cell IDs in sorted order for deterministic output
        let mut cell_ids: Vec<_> = layer.polygons.keys().collect();
        cell_ids.sort_by_key(|cell_id| cell_id.0);

        // Add all polygons in this layer in deterministic order
        for &cell_id in &cell_ids {
            if let Some(polygons) = layer.polygons.get(cell_id) {
                for polygon in polygons {
                    let path_data = polygon_to_path_data(polygon);
                    let path = Path::new().set("d", path_data).set("stroke", "none");
                    group = group.add(path);
                }
            }
        }

        doc = doc.add(group);
    }

    doc.to_string()
}

fn polygon_to_path_data(polygon: &geo::Polygon<f64>) -> String {
    let mut path_data = String::new();

    if let Some(point) = polygon.exterior().points().next() {
        path_data.push_str(&format!(
            "M {} {} ",
            round_to_precision(point.y()),
            round_to_precision(point.x())
        ));

        for point in polygon.exterior().points().skip(1) {
            path_data.push_str(&format!(
                "L {} {} ",
                round_to_precision(point.y()),
                round_to_precision(point.x())
            ));
        }
    }

    path_data.push('Z');

    // Add interior rings (holes)
    for interior in polygon.interiors() {
        if let Some(point) = interior.points().next() {
            path_data.push_str(&format!(
                "M {} {} ",
                round_to_precision(point.y()),
                round_to_precision(point.x())
            ));

            for point in interior.points().skip(1) {
                path_data.push_str(&format!(
                    "L {} {} ",
                    round_to_precision(point.y()),
                    round_to_precision(point.x())
                ));
            }

            path_data.push('Z');
        }
    }

    path_data
}

fn round_to_precision(value: f64) -> f64 {
    (value / PRECISION).round() * PRECISION
}
