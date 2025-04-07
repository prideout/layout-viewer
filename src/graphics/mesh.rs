use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use glow::HasContext;
use indexmap::IndexMap;
use nalgebra::Matrix4;
use nalgebra::Vector2;
use nalgebra::Vector3;
use nalgebra::Vector4;

use crate::graphics::geometry::Geometry;
use crate::graphics::material::Material;

#[derive(Component)]
pub struct Mesh {
    pub geometry: Entity,
    pub material: Entity,
    pub visible: bool,
    pub matrix: Matrix4<f32>,
    pub render_order: i32,
    float_uniforms: IndexMap<String, f32>,
    vec2_uniforms: IndexMap<String, Vector2<f32>>,
    vec3_uniforms: IndexMap<String, Vector3<f32>>,
    vec4_uniforms: IndexMap<String, Vector4<f32>>,
    mat4_uniforms: IndexMap<String, Matrix4<f32>>,
    int_uniforms: IndexMap<String, i32>,
    bool_uniforms: IndexMap<String, bool>,
}

#[allow(dead_code)]
impl Mesh {
    pub fn new(geometry: Entity, material: Entity) -> Self {
        Self {
            geometry,
            material,
            visible: true,
            matrix: Matrix4::identity(),
            render_order: 0,
            float_uniforms: IndexMap::new(),
            vec2_uniforms: IndexMap::new(),
            vec3_uniforms: IndexMap::new(),
            vec4_uniforms: IndexMap::new(),
            mat4_uniforms: IndexMap::new(),
            int_uniforms: IndexMap::new(),
            bool_uniforms: IndexMap::new(),
        }
    }

    pub fn set_float(&mut self, name: &str, value: f32) {
        self.float_uniforms.insert(name.to_string(), value);
    }

    pub fn set_vec2(&mut self, name: &str, value: Vector2<f32>) {
        self.vec2_uniforms.insert(name.to_string(), value);
    }

    pub fn set_vec3(&mut self, name: &str, value: Vector3<f32>) {
        self.vec3_uniforms.insert(name.to_string(), value);
    }

    pub fn set_vec4(&mut self, name: &str, value: Vector4<f32>) {
        self.vec4_uniforms.insert(name.to_string(), value);
    }

    pub fn set_mat4(&mut self, name: &str, value: Matrix4<f32>) {
        self.mat4_uniforms.insert(name.to_string(), value);
    }

    pub fn set_int(&mut self, name: &str, value: i32) {
        self.int_uniforms.insert(name.to_string(), value);
    }

    pub fn set_bool(&mut self, name: &str, value: bool) {
        self.bool_uniforms.insert(name.to_string(), value);
    }

    pub fn get_float(&self, name: &str) -> Option<&f32> {
        self.float_uniforms.get(name)
    }

    pub fn get_vec2(&self, name: &str) -> Option<&Vector2<f32>> {
        self.vec2_uniforms.get(name)
    }

    pub fn get_vec3(&self, name: &str) -> Option<&Vector3<f32>> {
        self.vec3_uniforms.get(name)
    }

    pub fn get_vec4(&self, name: &str) -> Option<&Vector4<f32>> {
        self.vec4_uniforms.get(name)
    }

    pub fn get_mat4(&self, name: &str) -> Option<&Matrix4<f32>> {
        self.mat4_uniforms.get(name)
    }

    pub fn get_int(&self, name: &str) -> Option<&i32> {
        self.int_uniforms.get(name)
    }

    pub fn get_bool(&self, name: &str) -> Option<&bool> {
        self.bool_uniforms.get(name)
    }

    pub fn draw(&self, gl: &glow::Context, material: &mut Material, geometry: &mut Geometry) {
        if geometry.indices.is_empty() {
            return;
        }
        material.set_mat4(gl, "model", &self.matrix);
        for (name, value) in &self.float_uniforms {
            material.set_float(gl, name, *value);
        }
        for (name, value) in &self.vec2_uniforms {
            material.set_vec2(gl, name, value);
        }
        for (name, value) in &self.vec3_uniforms {
            material.set_vec3(gl, name, value);
        }
        for (name, value) in &self.vec4_uniforms {
            material.set_vec4(gl, name, value);
        }
        for (name, value) in &self.mat4_uniforms {
            material.set_mat4(gl, name, value);
        }
        for (name, value) in &self.int_uniforms {
            material.set_int(gl, name, *value);
        }
        for (name, value) in &self.bool_uniforms {
            material.set_bool(gl, name, *value);
        }
        geometry.bind(gl);
        unsafe {
            gl.draw_elements(
                glow::TRIANGLES,
                geometry.indices.len() as i32,
                glow::UNSIGNED_INT,
                0,
            );
        }
    }
}

impl Default for Mesh {
    fn default() -> Self {
        Self::new(Entity::PLACEHOLDER, Entity::PLACEHOLDER)
    }
}
