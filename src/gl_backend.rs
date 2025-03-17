use geo::TriangulateEarcut;

use crate::gl_geometry::Geometry;
use crate::gl_material::Material;
use crate::gl_mesh::Mesh;
use crate::gl_scene::Scene;
use crate::layer::Layer;

#[cfg(target_arch = "wasm32")]
const VERTEX_SHADER: &str = r#"#version 100
attribute vec3 position;

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;
uniform vec4 color;

varying vec4 v_color;

void main() {
    gl_Position = projection * view * model * vec4(position, 1.0);
    v_color = color;
}
"#;

#[cfg(not(target_arch = "wasm32"))]
const VERTEX_SHADER: &str = r#"#version 330
layout (location = 0) in vec3 position;

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;
uniform vec4 color;

out vec4 v_color;

void main() {
    gl_Position = projection * view * model * vec4(position, 1.0);
    v_color = color;
}
"#;

#[cfg(target_arch = "wasm32")]
const FRAGMENT_SHADER: &str = r#"#version 100
precision mediump float;
varying vec4 v_color;

void main() {
    gl_FragColor = v_color;
}
"#;

#[cfg(not(target_arch = "wasm32"))]
const FRAGMENT_SHADER: &str = r#"#version 330
in vec4 v_color;
out vec4 FragColor;

void main() {
    FragColor = v_color;
}
"#;

pub fn populate_scene(layers: &[Layer], scene: &mut Scene) {
    let mut material = Material::new(VERTEX_SHADER, FRAGMENT_SHADER);

    material.set_blending(true);

    let material_id = scene.add_material(material);
    
    for layer in layers {
        let geometry = create_layer_geometry(layer);
        let geometry_id = scene.add_geometry(geometry);
        let mut mesh = Mesh::new(geometry_id, material_id);
        
        // Set the color uniform using the layer's color
        mesh.set_vec4("color", layer.color);
        
        scene.add_mesh(mesh);
    }
}

fn create_layer_geometry(layer: &Layer) -> Geometry {
    let mut geometry = Geometry::new();

    // Get layer bounds for normalization
    let layer_bounds = layer.bounds;
    
    // Calculate scale based on the larger dimension to ensure both fit in [-1, +1]
    let width_scale = 2.0 / layer_bounds.width();
    let height_scale = 2.0 / layer_bounds.height();
    let scale = width_scale.min(height_scale);

    // Calculate offsets to center the geometry
    let x_center = (layer_bounds.min_x + layer_bounds.max_x) / 2.0;
    let y_center = (layer_bounds.min_y + layer_bounds.max_y) / 2.0;

    // Process each polygon in the layer
    for cell_polygons in layer.polygons.values() {
        for polygon in cell_polygons {
            let triangles = polygon.earcut_triangles_raw();

            let vertex_offset = geometry.positions.len() as u32 / 3;

            for coord in triangles.vertices.chunks(2) {
                // Center the coordinates and then scale
                let x = (coord[0] - x_center) * scale;
                let y = (coord[1] - y_center) * scale;
                geometry
                    .positions
                    .extend_from_slice(&[y as f32, -x as f32, 0.0]);
            }

            geometry.indices.extend(
                triangles
                    .triangle_indices
                    .iter()
                    .map(|i| (*i as u32 + vertex_offset)),
            );
        }
    }

    geometry
}
