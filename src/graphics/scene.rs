use crate::graphics::{Geometry, GeometryId};
use crate::graphics::{Material, MaterialId};
use crate::graphics::{Mesh, MeshId};
use crate::id_map::IdMap;

pub struct Scene {
    pub(crate) meshes: IdMap<MeshId, Mesh>,
    pub(crate) geometries: IdMap<GeometryId, Geometry>,
    pub(crate) materials: IdMap<MaterialId, Material>,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            meshes: IdMap::new(),
            geometries: IdMap::new(),
            materials: IdMap::new(),
        }
    }

    pub fn add_mesh(&mut self, mesh: Mesh) -> MeshId {
        self.meshes.insert(mesh)
    }

    pub fn add_geometry(&mut self, geometry: Geometry) -> GeometryId {
        self.geometries.insert(geometry)
    }

    pub fn add_material(&mut self, material: Material) -> MaterialId {
        self.materials.insert(material)
    }

    pub fn get_mesh(&self, id: &MeshId) -> Option<&Mesh> {
        self.meshes.get(id)
    }

    pub fn get_mesh_mut(&mut self, id: &MeshId) -> Option<&mut Mesh> {
        self.meshes.get_mut(id)
    }

    pub fn get_geometry(&self, id: &GeometryId) -> Option<&Geometry> {
        self.geometries.get(id)
    }

    pub fn get_geometry_mut(&mut self, id: &GeometryId) -> Option<&mut Geometry> {
        self.geometries.get_mut(id)
    }

    pub fn get_material(&self, id: &MaterialId) -> Option<&Material> {
        self.materials.get(id)
    }

    pub fn get_material_mut(&mut self, id: &MaterialId) -> Option<&mut Material> {
        self.materials.get_mut(id)
    }

    pub fn destroy(&mut self, gl: &glow::Context) {
        // Destroy all geometries
        for geometry in self.geometries.values_mut() {
            geometry.destroy(gl);
        }
        self.geometries.clear();

        // Destroy all materials
        for material in self.materials.values_mut() {
            material.destroy(gl);
        }
        self.materials.clear();

        // Clear meshes (they don't own any GL resources)
        self.meshes.clear();
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self::new()
    }
}
