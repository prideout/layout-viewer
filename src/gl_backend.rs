use crate::graphics::Geometry;
use crate::graphics::Material;
use crate::graphics::Mesh;
use crate::graphics::Scene;
use crate::layer::Layer;
use geo::TriangulateEarcut;

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

/// Triangulates polygons and appends them to a vertex buffer.
fn create_layer_geometry(layer: &Layer) -> Geometry {
    let mut geometry = Geometry::new();

    // Process each polygon in the layer
    for polygon in &layer.polygons {
        let triangles = polygon.earcut_triangles_raw();

        let vertex_offset = geometry.positions.len() as u32 / 3;

        for coord in triangles.vertices.chunks(2) {
            let x = coord[0];
            let y = coord[1];
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

    geometry
}
