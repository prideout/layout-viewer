#![allow(dead_code)]

use nalgebra::{Matrix4, Point3, Vector3};
use crate::id_map::{Id, IdMap};
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

pub struct Mesh;
pub struct Geometry;
pub struct Material;

impl Id for ViewId {
    fn from_usize(id: usize) -> Self { ViewId(id) }
}

impl Id for CameraId {
    fn from_usize(id: usize) -> Self { CameraId(id) }
}

impl Id for MeshId {
    fn from_usize(id: usize) -> Self { MeshId(id) }
}

impl Id for GeometryId {
    fn from_usize(id: usize) -> Self { GeometryId(id) }
}

impl Id for MaterialId {
    fn from_usize(id: usize) -> Self { MaterialId(id) }
}

pub struct GlScene {
    views: IdMap<ViewId, View>,
    cameras: IdMap<CameraId, Camera>,
    meshes: IdMap<MeshId, Mesh>,
    geometries: IdMap<GeometryId, Geometry>,
    materials: IdMap<MaterialId, Material>,
}

impl GlScene {
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