use crate::graphics::Material;
use crate::graphics::MaterialId;
use crate::graphics::Mesh;
use crate::graphics::MeshId;
use crate::rsutils::IdMap;

pub struct Scene {
    pub(super) meshes: IdMap<MeshId, Mesh>,
    pub(super) materials: IdMap<MaterialId, Material>,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            meshes: IdMap::new(),
            materials: IdMap::new(),
        }
    }

    pub fn add_mesh(&mut self, mesh: Mesh) -> MeshId {
        self.meshes.insert(mesh)
    }

    pub fn add_material(&mut self, material: Material) -> MaterialId {
        self.materials.insert(material)
    }

    #[allow(dead_code)]
    pub fn get_mesh(&self, id: &MeshId) -> Option<&Mesh> {
        self.meshes.get(id)
    }

    pub fn get_mesh_mut(&mut self, id: &MeshId) -> Option<&mut Mesh> {
        self.meshes.get_mut(id)
    }

    #[allow(dead_code)]
    pub fn get_material(&self, id: &MaterialId) -> Option<&Material> {
        self.materials.get(id)
    }

    pub fn get_material_mut(&mut self, id: &MaterialId) -> Option<&mut Material> {
        self.materials.get_mut(id)
    }

    pub fn move_mesh_to_back(&mut self, id: MeshId) {
        if self.meshes.get_index_of(id).is_some() {
            self.meshes.move_to_back(id);
        } else {
            log::error!("Scene: move_mesh_to_back called with non-existent id");
        }
    }

    pub fn destroy(&mut self, gl: &glow::Context) {
        for material in self.materials.values_mut() {
            material.destroy(gl);
        }
        self.materials.clear();
        self.meshes.clear();
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self::new()
    }
}
