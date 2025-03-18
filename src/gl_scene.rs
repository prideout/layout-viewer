use crate::cells::CellId;
use crate::gl_geometry::{Geometry, GeometryId};
use crate::gl_material::{Material, MaterialId};
use crate::gl_mesh::{Mesh, MeshId};
use crate::id_map::IdMap;

pub struct Scene {
    pub(crate) meshes: IdMap<MeshId, Mesh>,
    pub(crate) geometries: IdMap<GeometryId, Geometry>,
    pub(crate) materials: IdMap<MaterialId, Material>,
    pub(crate) triangle_info: Vec<TriangleInfo>,
}

/// Each triangle in the scene has a TriangleInfo that tells the app which cell
/// and layer the triangle belongs to. Useful for hovering and selecting.
pub struct TriangleInfo {
    pub cell_id: u32,
    pub layer_index: i16,
    pub padding: u16,
}

impl TriangleInfo {
    pub fn new(cell_id: CellId, layer_index: i16) -> Self {
        Self {
            cell_id: cell_id.0 as u32,
            layer_index,
            padding: 0,
        }
    }
}

impl Scene {
    pub fn new() -> Self {
        Self {
            meshes: IdMap::new(),
            geometries: IdMap::new(),
            materials: IdMap::new(),
            triangle_info: Vec::new(),
        }
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
