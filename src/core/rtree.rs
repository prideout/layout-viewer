use bevy_ecs::entity::Entity;
use rstar::Envelope;
use rstar::PointDistance;
use rstar::RTreeObject;
use rstar::AABB;

#[derive(Clone)]
pub struct RTreeItem {
    pub shape_instance: Entity,
    pub aabb: AABB<geo::Point<f64>>,
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
