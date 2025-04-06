use super::default_shaders::FRAGMENT_SHADER;
use super::default_shaders::VERTEX_SHADER;
use bevy_ecs::component::Component;
use glow::HasContext;
use indexmap::IndexMap;
use nalgebra::Matrix4;
use nalgebra::Vector2;
use nalgebra::Vector3;
use nalgebra::Vector4;
use std::hash::Hash;

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub enum BlendMode {
    Disabled,
    SourceOver,
    Additive,
}

#[derive(Component)]
pub struct Material {
    program: Option<glow::Program>,
    uniform_locations: IndexMap<String, glow::UniformLocation>,
    vertex_shader: String,
    fragment_shader: String,
    blend_mode: BlendMode,
    depth_test: bool,
}

// SAFETY: This is safe because:
// 1. The app is single-threaded
// 2. OpenGL resources are only accessed from the render thread
// 3. The uniform locations are only modified during program creation
unsafe impl Send for Material {}
unsafe impl Sync for Material {}

impl Material {
    pub fn new(vertex_shader: &str, fragment_shader: &str) -> Self {
        Self {
            program: None,
            uniform_locations: IndexMap::new(),
            vertex_shader: vertex_shader.to_string(),
            fragment_shader: fragment_shader.to_string(),
            blend_mode: BlendMode::Disabled,
            depth_test: false,
        }
    }

    pub fn set_blending(&mut self, mode: BlendMode) {
        self.blend_mode = mode;
    }

    #[allow(dead_code)]
    pub fn blend_mode(&self) -> BlendMode {
        self.blend_mode
    }

    pub(crate) fn create_program(&mut self, gl: &glow::Context) {
        if self.program.is_some() {
            unsafe {
                gl.delete_program(self.program.unwrap());
            }
        }

        unsafe {
            let program = gl.create_program().unwrap();

            // Create and compile shaders
            let shader_sources = [
                (glow::VERTEX_SHADER, &self.vertex_shader),
                (glow::FRAGMENT_SHADER, &self.fragment_shader),
            ];

            let mut shaders = Vec::with_capacity(shader_sources.len());

            for (shader_type, shader_source) in shader_sources.iter() {
                let shader = gl
                    .create_shader(*shader_type)
                    .expect("Cannot create shader");
                gl.shader_source(shader, shader_source);
                gl.compile_shader(shader);
                if !gl.get_shader_compile_status(shader) {
                    let slog = gl.get_shader_info_log(shader);
                    log::error!("Shader compilation failed: {}", slog);
                    panic!("Shader compilation failed");
                }
                gl.attach_shader(program, shader);
                shaders.push(shader);
            }

            // Link program
            gl.link_program(program);
            if !gl.get_program_link_status(program) {
                let log = gl.get_program_info_log(program);
                log::error!("Program linking failed: {}", log);
                panic!("Program linking failed");
            }

            // Clean up shaders
            for shader in shaders {
                gl.detach_shader(program, shader);
                gl.delete_shader(shader);
            }

            self.program = Some(program);
            self.gather_uniforms(gl);
        }
    }

    pub(crate) fn gather_uniforms(&mut self, gl: &glow::Context) {
        let Some(program) = self.program else { return };
        self.uniform_locations.clear();

        unsafe {
            let count = gl.get_active_uniforms(program);
            for i in 0..count {
                if let Some(info) = gl.get_active_uniform(program, i) {
                    if let Some(location) = gl.get_uniform_location(program, &info.name) {
                        self.uniform_locations.insert(info.name, location);
                    }
                }
            }
        }
    }

    pub(crate) fn destroy(&mut self, gl: &glow::Context) {
        if let Some(program) = self.program.take() {
            unsafe {
                gl.delete_program(program);
            }
        }
        self.uniform_locations.clear();
    }

    pub(crate) fn bind(&mut self, gl: &glow::Context) {
        if self.program.is_none() {
            self.create_program(gl);
        }

        unsafe {
            gl.use_program(self.program);

            if self.depth_test {
                gl.enable(glow::DEPTH_TEST);
            } else {
                gl.disable(glow::DEPTH_TEST);
            }

            match self.blend_mode {
                BlendMode::Disabled => {
                    gl.disable(glow::BLEND);
                }
                BlendMode::SourceOver => {
                    gl.enable(glow::BLEND);
                    gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
                }
                BlendMode::Additive => {
                    gl.enable(glow::BLEND);
                    gl.blend_func(glow::ONE, glow::ONE);
                }
            }
        }
    }

    pub(crate) fn set_float(&mut self, gl: &glow::Context, name: &str, value: f32) {
        if let Some(location) = self.uniform_locations.get(name) {
            unsafe {
                gl.uniform_1_f32(Some(location), value);
            }
        } else {
            log::warn!(
                "Attempting to set unknown uniform '{}' to float value",
                name
            );
        }
    }

    pub(crate) fn set_vec2(&mut self, gl: &glow::Context, name: &str, value: &Vector2<f32>) {
        if let Some(location) = self.uniform_locations.get(name) {
            unsafe {
                gl.uniform_2_f32(Some(location), value.x, value.y);
            }
        } else {
            log::warn!("Attempting to set unknown uniform '{}' to vec2 value", name);
        }
    }

    pub(crate) fn set_vec3(&mut self, gl: &glow::Context, name: &str, value: &Vector3<f32>) {
        if let Some(location) = self.uniform_locations.get(name) {
            unsafe {
                gl.uniform_3_f32(Some(location), value.x, value.y, value.z);
            }
        } else {
            log::warn!("Attempting to set unknown uniform '{}' to vec3 value", name);
        }
    }

    pub(crate) fn set_vec4(&mut self, gl: &glow::Context, name: &str, value: &Vector4<f32>) {
        if let Some(location) = self.uniform_locations.get(name) {
            unsafe {
                gl.uniform_4_f32(Some(location), value.x, value.y, value.z, value.w);
            }
        } else {
            log::warn!("Attempting to set unknown uniform '{}' to vec4 value", name);
        }
    }

    pub(crate) fn set_mat4(&mut self, gl: &glow::Context, name: &str, value: &Matrix4<f32>) {
        if let Some(location) = self.uniform_locations.get(name) {
            unsafe {
                gl.uniform_matrix_4_f32_slice(Some(location), false, value.as_slice());
            }
        } else {
            log::warn!("Attempting to set unknown uniform '{}' to mat4 value", name);
        }
    }

    pub(crate) fn set_int(&mut self, gl: &glow::Context, name: &str, value: i32) {
        if let Some(location) = self.uniform_locations.get(name) {
            unsafe {
                gl.uniform_1_i32(Some(location), value);
            }
        } else {
            log::warn!("Attempting to set unknown uniform '{}' to int value", name);
        }
    }

    pub(crate) fn set_bool(&mut self, gl: &glow::Context, name: &str, value: bool) {
        if let Some(location) = self.uniform_locations.get(name) {
            unsafe {
                gl.uniform_1_i32(Some(location), value as i32);
            }
        } else {
            log::warn!("Attempting to set unknown uniform '{}' to bool value", name);
        }
    }
}

impl Drop for Material {
    fn drop(&mut self) {
        if self.program.is_some() {
            log::warn!("Material dropped without calling destroy()");
        }
    }
}

impl Default for Material {
    fn default() -> Self {
        Material::new(VERTEX_SHADER, FRAGMENT_SHADER)
    }
}
