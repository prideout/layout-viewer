#![allow(unused)]

use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use geo::AffineTransform;

use crate::graphics::BoundingBox;

type Point2d = nalgebra::Point2<f64>;
type Vector4f = nalgebra::Vector4<f32>;
type Point2f = nalgebra::Point2<f32>;
type Polygon = geo::Polygon<f64>;

#[derive(Component)]
pub struct CellDefinition {
    pub name: String,
    pub shape_refs: Vec<ShapeReference>,
    pub cell_refs: Vec<CellReference>,

    /// Users must choose a cell definition to be the active view context.
    ///
    /// When this choice is made, a new tree of instances are created, and the
    /// chosen cell definition is used to instantiate the root.
    ///
    /// The root instantiation is stored here, but only if this cell definition
    /// happens to be the chosen one. For all other definitions, this is `None`.
    pub root_instance: Option<CellInstance>,
}

#[derive(Component)]
pub struct CellInstance {
    pub cell_definition: Entity,

    /// Must have same length as CellDefinition::shape_refs
    pub shape_instances: Vec<Entity>,

    /// Must have same length as CellDefinition::cell_refs
    pub child_instances: Vec<Entity>,

    /// Transforms this cell's coord system to the root coord system.
    pub world_transform: AffineTransform,
}

#[derive(Component)]
pub struct ShapeDefinition {
    pub layer: Entity,
    pub shape_type: ShapeType,
    pub local_polygon: Polygon,
    pub local_triangles: Triangulation,
}

// The rtree item has:
//  (1) this entity id
//  (2) the aabb of this world_polygon.
#[derive(Component)]
pub struct ShapeInstance {
    pub cell_instance: Entity,
    pub world_polygon: Polygon,
}

#[derive(Component)]
pub struct Layer {
    pub index: u16,
    pub color: Vector4f,
    pub visible: bool,
    pub mesh: Entity,
    pub world_bounds: BoundingBox,
    pub shape_instances: Vec<Entity>,
}

pub struct ShapeReference {
    pub shape_definition: Entity,
    pub local_transform: AffineTransform,
}

pub struct CellReference {
    pub cell_definition: Entity,
    pub local_transform: AffineTransform,
}

pub enum ShapeType {
    Polygon(Vec<Point2d>),
    Path { width: f64, spine: Vec<Point2d> },
}

pub struct Triangulation {
    pub indices: Vec<u32>,
    pub vertices: Vec<Point2f>,
}

impl Triangulation {
    pub fn empty() -> Self {
        Self { indices: vec![], vertices: vec![] }
    }
}

