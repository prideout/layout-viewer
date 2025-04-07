use bevy_ecs::entity::Entity;

use crate::core::components::Layer;
use crate::rsutils::colors::hex_to_rgb;
use crate::rsutils::colors::rgb_to_hex;

/// Represents a layer in the sidebar.
#[derive(Clone, PartialEq)]
pub struct LayerProxy {
    pub entity: Entity,
    pub index: i16,
    pub visible: bool,
    pub opacity: f32,
    pub color: String,
    pub is_empty: bool,
}

impl LayerProxy {
    pub fn from_layer(entity: Entity, layer: &Layer) -> Self {
        Self {
            entity,
            index: layer.index,
            visible: layer.visible,
            opacity: layer.color.w,
            color: rgb_to_hex(layer.color.x, layer.color.y, layer.color.z),
            is_empty: layer.shape_instances.is_empty(),
        }
    }

    pub fn to_layer(&self, layer: &mut Layer) {
        layer.visible = self.visible;
        layer.color.w = self.opacity;
        let rgb = hex_to_rgb(&self.color).unwrap();
        layer.color.x = rgb.0;
        layer.color.y = rgb.1;
        layer.color.z = rgb.2;
    }
}
