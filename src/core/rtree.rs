use bevy_ecs::entity::Entity;
use bevy_ecs::world::World;
use rstar::Envelope;
use rstar::PointDistance;
use rstar::RTree;
use rstar::RTreeObject;
use rstar::AABB;

use crate::app_shaders::FRAGMENT_SHADER;
use crate::app_shaders::VERTEX_SHADER;
use crate::graphics::Camera;
use crate::graphics::Geometry;
use crate::graphics::Material;
use crate::graphics::Renderer;
use crate::graphics::Viewport;
use crate::hover_effect::HoverEffect;

type Point3 = nalgebra::Point3<f64>;

#[derive(Clone)]
pub struct RTreeItem {
    pub shape_instance: Entity,
    pub aabb: AABB<geo::Point<f64>>,
    pub layer: i16,
}

impl PartialEq for RTreeItem {
    fn eq(&self, other: &Self) -> bool {
        self.shape_instance == other.shape_instance
    }
}

impl Eq for RTreeItem {}

impl RTreeObject for RTreeItem {
    type Envelope = AABB<geo::Point<f64>>;

    fn envelope(&self) -> Self::Envelope {
        self.aabb
    }
}

impl PointDistance for RTreeItem {
    fn distance_2(&self, point: &geo::Point<f64>) -> f64 {
        self.aabb.distance_2(point)
    }

    fn contains_point(&self, point: &geo::Point<f64>) -> bool {
        self.aabb.contains_point(point)
    }
}
