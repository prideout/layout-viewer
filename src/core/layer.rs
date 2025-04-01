use crate::core::CellDefId;
use crate::graphics::BoundingBox;
use crate::graphics::MeshId;
use geo::AffineOps;
use geo::AffineTransform;
use geo::BoundingRect;
use nalgebra::Vector4;

use super::CellId;

type Polygon = geo::Polygon<f64>;

pub struct ElementInstance {
    pub polygon: Polygon,
    pub cell_def_id: CellDefId,
    pub element_index: usize,
    pub cell_id: CellId,
}

pub struct Layer {
    index: i16,
    pub mesh: Option<MeshId>,
    pub element_instances: Vec<ElementInstance>,
    pub bounds: BoundingBox,
    pub color: Vector4<f32>, // RGBA color for this layer
    pub visible: bool,
}

impl Layer {
    pub fn new(index: i16) -> Self {
        Self {
            index,
            mesh: None,
            element_instances: vec![],
            bounds: BoundingBox::new(),
            color: Vector4::new(0.0, 0.0, 0.0, 1.0), // Default to black
            visible: true,
        }
    }

    pub fn index(&self) -> i16 {
        self.index
    }

    pub fn update_bounds(&mut self) {
        self.bounds = BoundingBox::new();

        for element_instance in &self.element_instances {
            if let Some(bbox) = element_instance.polygon.bounding_rect() {
                let layer_bbox = BoundingBox::from(bbox);
                self.bounds.encompass(&layer_bbox);
            }
        }
    }

    pub fn add_element_instance(
        &mut self,
        mut element_instance: ElementInstance,
        transform: &AffineTransform,
    ) {
        element_instance.polygon = element_instance.polygon.affine_transform(transform);
        self.element_instances.push(element_instance);
    }
}
