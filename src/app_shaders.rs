#[cfg(target_arch = "wasm32")]
pub const VERTEX_SHADER: &str = r#"#version 100
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
pub const VERTEX_SHADER: &str = r#"#version 330
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
pub const FRAGMENT_SHADER: &str = r#"#version 100
precision mediump float;
varying vec4 v_color;

void main() {
    gl_FragColor = v_color;
}
"#;

#[cfg(not(target_arch = "wasm32"))]
pub const FRAGMENT_SHADER: &str = r#"#version 330
in vec4 v_color;
out vec4 FragColor;

void main() {
    FragColor = v_color;
}
"#;
