use crate::graphics::Material;
use crate::graphics::MaterialId;
use crate::rsutils::IdMap;

pub struct Scene {
    pub(super) materials: IdMap<MaterialId, Material>,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            materials: IdMap::new(),
        }
    }

    pub fn add_material(&mut self, material: Material) -> MaterialId {
        self.materials.insert(material)
    }

    #[allow(dead_code)]
    pub fn get_material(&self, id: &MaterialId) -> Option<&Material> {
        self.materials.get(id)
    }

    pub fn get_material_mut(&mut self, id: &MaterialId) -> Option<&mut Material> {
        self.materials.get_mut(id)
    }

    pub fn destroy(&mut self, gl: &glow::Context) {
        for material in self.materials.values_mut() {
            material.destroy(gl);
        }
        self.materials.clear();
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self::new()
    }
}
