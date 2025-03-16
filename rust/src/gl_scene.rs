#![allow(dead_code)]

use crate::id_map::{Id, IdMap};
use glow::HasContext;
use nalgebra::{Matrix4, Point3, Vector3};
use std::hash::Hash;

type Mat4 = Matrix4<f32>;
type Vec3 = Vector3<f32>;
type Point = Point3<f32>;

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct ViewId(pub usize);

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct CameraId(pub usize);

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct MeshId(pub usize);

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct GeometryId(pub usize);

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct MaterialId(pub usize);

pub struct View;

pub struct Camera {
    pub position: Point,
    pub width: f32,
    pub height: f32,
    pub near: f32,
    pub far: f32,
}

impl Camera {
    pub fn new(position: Point, width: f32, height: f32, near: f32, far: f32) -> Self {
        Self {
            position,
            width,
            height,
            near,
            far,
        }
    }

    pub fn get_projection_matrix(&self) -> Mat4 {
        let half_width = self.width / 2.0;
        let half_height = self.height / 2.0;

        Mat4::new_orthographic(
            -half_width,  // left
            half_width,   // right
            -half_height, // bottom
            half_height,  // top
            self.near,    // near
            self.far,     // far
        )
    }
}

pub struct Geometry {
    vao: Option<glow::VertexArray>,
    positions_vbo: Option<glow::Buffer>,
    indices_vbo: Option<glow::Buffer>,
}

impl Geometry {
    pub fn new() -> Self {
        Self {
            vao: None,
            positions_vbo: None,
            indices_vbo: None,
        }
    }

    pub fn create(&mut self, gl: &glow::Context) {
        unsafe {
            self.vao = Some(gl.create_vertex_array().expect("Failed to create VAO"));
            self.positions_vbo = Some(gl.create_buffer().expect("Failed to create positions VBO"));
            self.indices_vbo = Some(gl.create_buffer().expect("Failed to create indices VBO"));
        }
    }

    pub fn destroy(&self, gl: &glow::Context) {
        unsafe {
            gl.delete_vertex_array(self.vao.unwrap());
            gl.delete_buffer(self.positions_vbo.unwrap());
            gl.delete_buffer(self.indices_vbo.unwrap());
        }
    }

    pub fn upload_positions(&self, gl: &glow::Context, positions: &[f32]) {
        unsafe {
            gl.bind_vertex_array(Some(self.vao.unwrap()));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.positions_vbo.unwrap()));
            gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                bytemuck::cast_slice(positions),
                glow::STATIC_DRAW,
            );
            gl.enable_vertex_attrib_array(0);
            gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, 12, 0);
        }
    }

    pub fn upload_indices(&self, gl: &glow::Context, indices: &[u32]) {
        unsafe {
            gl.bind_vertex_array(Some(self.vao.unwrap()));
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(self.indices_vbo.unwrap()));
            gl.buffer_data_u8_slice(
                glow::ELEMENT_ARRAY_BUFFER,
                bytemuck::cast_slice(indices),
                glow::STATIC_DRAW,
            );
        }
    }

    pub fn bind(&self, gl: &glow::Context) {
        unsafe {
            gl.bind_vertex_array(self.vao);
        }
    }
}

impl Drop for Geometry {
    fn drop(&mut self) {
        if self.vao.is_some() {
            eprintln!("Warning: Geometry dropped without calling destroy()");
        }
    }
}

pub struct Mesh;
pub struct Material;

impl Id for ViewId {
    fn from_usize(id: usize) -> Self {
        ViewId(id)
    }
}

impl Id for CameraId {
    fn from_usize(id: usize) -> Self {
        CameraId(id)
    }
}

impl Id for MeshId {
    fn from_usize(id: usize) -> Self {
        MeshId(id)
    }
}

impl Id for GeometryId {
    fn from_usize(id: usize) -> Self {
        GeometryId(id)
    }
}

impl Id for MaterialId {
    fn from_usize(id: usize) -> Self {
        MaterialId(id)
    }
}

pub struct Scene {
    views: IdMap<ViewId, View>,
    cameras: IdMap<CameraId, Camera>,
    meshes: IdMap<MeshId, Mesh>,
    geometries: IdMap<GeometryId, Geometry>,
    materials: IdMap<MaterialId, Material>,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            views: IdMap::new(),
            cameras: IdMap::new(),
            meshes: IdMap::new(),
            geometries: IdMap::new(),
            materials: IdMap::new(),
        }
    }

    pub fn add_camera(&mut self, camera: Camera) -> CameraId {
        self.cameras.create_id(camera)
    }

    pub fn add_view(&mut self, view: View) -> ViewId {
        self.views.create_id(view)
    }

    pub fn add_mesh(&mut self, mesh: Mesh) -> MeshId {
        self.meshes.create_id(mesh)
    }

    pub fn add_geometry(&mut self, geometry: Geometry) -> GeometryId {
        self.geometries.create_id(geometry)
    }

    pub fn add_material(&mut self, material: Material) -> MaterialId {
        self.materials.create_id(material)
    }
}
