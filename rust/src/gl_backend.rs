use crate::gl_geometry::Geometry;
use crate::gl_material::Material;
use crate::gl_mesh::Mesh;
use crate::gl_scene::Scene;
use crate::render_layer::RenderLayer;
use geo::TriangulateEarcut;

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

const FRAGMENT_SHADER: &str = r#"#version 330
in vec4 v_color;
out vec4 FragColor;

void main() {
    FragColor = v_color;
}
"#;

pub fn populate_scene(layers: &[RenderLayer], scene: &mut Scene) {
    let material = Material::new(VERTEX_SHADER, FRAGMENT_SHADER);
    let material_id = scene.add_material(material);
    for layer in layers {
        let geometry = create_layer_geometry(layer);
        let geometry_id = scene.add_geometry(geometry);
        let mesh = Mesh::new(geometry_id, material_id);
        scene.add_mesh(mesh);
    }
}

fn create_layer_geometry(layer: &RenderLayer) -> Geometry {
    let mut geometry = Geometry::new();

    // Get layer bounds for normalization
    let layer_bounds = layer.bounds;
    let scale = 2.0 / layer_bounds.width(); // Map to [-1, +1] along X axis
    let x_offset = -1.0 - (layer_bounds.min_x * scale);
    let y_scale = scale; // Maintain aspect ratio

    // Process each polygon in the layer
    for cell_polygons in layer.polygons.values() {
        for polygon in cell_polygons {
            let triangles = polygon.earcut_triangles_raw();

            let vertex_offset = geometry.positions.len() as u32 / 3;

            for coord in triangles.vertices.chunks(2) {
                let x = coord[0] * scale + x_offset;
                let y = coord[1] * y_scale;
                geometry
                    .positions
                    .extend_from_slice(&[x as f32, y as f32, 0.0]);
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
