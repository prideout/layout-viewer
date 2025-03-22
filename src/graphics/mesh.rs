use crate::graphics::Geometry;
use crate::graphics::GeometryId;
use crate::graphics::Material;
use crate::graphics::MaterialId;
use crate::rsutils::IdMapKey;
use glow::HasContext;
use indexmap::IndexMap;
use nalgebra::Matrix4;
use nalgebra::Vector2;
use nalgebra::Vector3;
use nalgebra::Vector4;
use std::hash::Hash;

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct MeshId(pub usize);

impl IdMapKey for MeshId {
    fn from_usize(id: usize) -> Self {
        MeshId(id)
    }
}

pub struct Mesh {
    pub geometry_id: GeometryId,
    pub material_id: MaterialId,
    pub visible: bool,
    pub matrix: Matrix4<f32>,
    float_uniforms: IndexMap<String, f32>,
    vec2_uniforms: IndexMap<String, Vector2<f32>>,
    vec3_uniforms: IndexMap<String, Vector3<f32>>,
    vec4_uniforms: IndexMap<String, Vector4<f32>>,
    mat4_uniforms: IndexMap<String, Matrix4<f32>>,
    int_uniforms: IndexMap<String, i32>,
    bool_uniforms: IndexMap<String, bool>,
}

impl Mesh {
    pub fn new(geometry_id: GeometryId, material_id: MaterialId) -> Self {
        Self {
            geometry_id,
            material_id,
            visible: true,
            matrix: Matrix4::identity(),
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
        if !self.visible {
            return;
        }

        // Do not emit a warning here, we already did.
        if geometry.indices.is_empty() {
            return;
        }

        // Assumes material is already bound.

        unsafe {
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

            gl.draw_elements(
                glow::TRIANGLES,
                geometry.indices.len() as i32,
                glow::UNSIGNED_INT,
                0,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_float_uniform() {
        let geom_id = GeometryId(0);
        let mat_id = MaterialId(0);
        let mut mesh = Mesh::new(geom_id, mat_id);

        mesh.set_float("scale", 2.5);
        assert_relative_eq!(*mesh.get_float("scale").unwrap(), 2.5);
        assert!(mesh.get_float("nonexistent").is_none());
    }

    #[test]
    fn test_vec2_uniform() {
        let geom_id = GeometryId(0);
        let mat_id = MaterialId(0);
        let mut mesh = Mesh::new(geom_id, mat_id);

        // Test basic set/get
        let pos = Vector2::new(1.5, -2.3);
        mesh.set_vec2("position", pos);
        let retrieved = mesh.get_vec2("position").unwrap();
        assert_relative_eq!(retrieved.x, 1.5);
        assert_relative_eq!(retrieved.y, -2.3);

        // Test overwriting
        let new_pos = Vector2::new(0.0, 1.0);
        mesh.set_vec2("position", new_pos);
        let retrieved = mesh.get_vec2("position").unwrap();
        assert_relative_eq!(retrieved.x, 0.0);
        assert_relative_eq!(retrieved.y, 1.0);

        // Test multiple vec2s
        let scale = Vector2::new(2.0, 3.0);
        mesh.set_vec2("scale", scale);
        assert_relative_eq!(mesh.get_vec2("position").unwrap(), &new_pos);
        assert_relative_eq!(mesh.get_vec2("scale").unwrap(), &scale);

        // Test nonexistent
        assert!(mesh.get_vec2("nonexistent").is_none());
    }

    #[test]
    fn test_vector_uniforms() {
        let geom_id = GeometryId(0);
        let mat_id = MaterialId(0);
        let mut mesh = Mesh::new(geom_id, mat_id);

        let vec2 = Vector2::new(1.0, 2.0);
        let vec3 = Vector3::new(1.0, 2.0, 3.0);
        let vec4 = Vector4::new(1.0, 2.0, 3.0, 4.0);

        mesh.set_vec2("offset", vec2);
        mesh.set_vec3("color", vec3);
        mesh.set_vec4("quaternion", vec4);

        assert_relative_eq!(mesh.get_vec2("offset").unwrap(), &vec2);
        assert_relative_eq!(mesh.get_vec3("color").unwrap(), &vec3);
        assert_relative_eq!(mesh.get_vec4("quaternion").unwrap(), &vec4);
    }

    #[test]
    fn test_matrix_uniform() {
        let geom_id = GeometryId(0);
        let mat_id = MaterialId(0);
        let mut mesh = Mesh::new(geom_id, mat_id);

        let mat = Matrix4::new_scaling(2.0);
        mesh.set_mat4("transform", mat);
        assert_relative_eq!(mesh.get_mat4("transform").unwrap(), &mat);
    }

    #[test]
    fn test_primitive_uniforms() {
        let geom_id = GeometryId(0);
        let mat_id = MaterialId(0);
        let mut mesh = Mesh::new(geom_id, mat_id);

        mesh.set_int("count", 42);
        mesh.set_bool("enabled", true);

        assert_eq!(*mesh.get_int("count").unwrap(), 42);
        assert!(*mesh.get_bool("enabled").unwrap());
    }

    #[test]
    fn test_overwrite_uniform() {
        let geom_id = GeometryId(0);
        let mat_id = MaterialId(0);
        let mut mesh = Mesh::new(geom_id, mat_id);

        mesh.set_float("value", 1.0);
        assert_relative_eq!(*mesh.get_float("value").unwrap(), 1.0);

        mesh.set_float("value", 2.0);
        assert_relative_eq!(*mesh.get_float("value").unwrap(), 2.0);
    }
}
