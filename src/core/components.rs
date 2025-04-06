#![allow(unused)]

use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use geo::AffineOps;
use geo::AffineTransform;

use crate::graphics::BoundingBox;
use crate::graphics::Geometry;
use crate::graphics::Material;

pub type Point2d = nalgebra::Point2<f64>;
pub type Vector4f = nalgebra::Vector4<f32>;
pub type Point2f = nalgebra::Point2<f32>;
pub type Polygon = geo::Polygon<f64>;

#[derive(Component)]
pub struct Selected;

#[derive(Component)]
pub struct Hovered;

/// Marker for the singleton CellInstance at the root of the instance tree.
///
/// At run time, users can choose any cell definition to be the active view
/// context. When this choice is made, a new tree of instances are created, and
/// the chosen cell definition is used to instantiate the root.
#[derive(Component)]
#[require(CellInstance)]
pub struct RootCellInstance;

#[derive(Component)]
pub struct CellDefinition {
    pub name: String,
    pub shape_defs: Vec<Entity>,
    pub cell_refs: Vec<CellReference>,
}

#[derive(Component)]
pub struct CellInstance {
    pub cell_definition: Entity,

    /// Must have same length as CellDefinition::shape_defs
    pub shape_instances: Vec<Entity>,

    /// Must have same length as CellDefinition::cell_refs
    pub child_instances: Vec<Entity>,

    /// Transforms this cell's coord system to the root coord system.
    pub world_transform: AffineTransform,
    // NOTE: consider storing a GeometryRange here for fast VBO updates.
}

#[derive(Component)]
pub struct ShapeDefinition {
    pub layer: Entity,
    pub shape_type: ShapeType,
    pub local_polygon: Polygon,
    pub local_triangles: Triangulation,
}

// NOTE: We use an R-tree for fast spatial lookups. Each node in the tree has:
//  (1) this entity id
//  (2) the aabb of this world_polygon
#[derive(Component)]
pub struct ShapeInstance {
    pub cell_instance: Entity,
    pub world_polygon: Polygon,
    pub layer_index: i16,
}

#[derive(Component)]
pub struct Layer {
    pub index: i16,
    pub color: Vector4f,
    pub visible: bool,
    pub mesh: Entity,
    pub world_bounds: BoundingBox,
    pub shape_instances: Vec<Entity>,
}

/// Marker for the singleton Material shared across all layer meshes.
#[derive(Component)]
#[require(Material)]
pub struct LayerMaterial;

#[derive(Clone)]
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
        Self {
            indices: vec![],
            vertices: vec![],
        }
    }

    // TODO: Make this more streamlined by taking an f32 AffineTransform and avoiding
    // the back-and-forth conversion to geo::Point.
    pub fn affine_transform(&self, transform: &AffineTransform) -> Self {
        let indices = self.indices.clone();
        let vertices = self
            .vertices
            .iter()
            .map(|v| from_geo(to_geo(v).affine_transform(transform)))
            .collect();
        Self { indices, vertices }
    }

    pub fn append_to(&self, geo: &mut Geometry) {
        let start_index = (geo.positions.len() / 3) as u32;
        for vert in &self.vertices {
            geo.positions.push(vert.x);
            geo.positions.push(vert.y);
            geo.positions.push(0.0);
        }
        for index in &self.indices {
            geo.indices.push(start_index + *index);
        }
    }
}

fn to_geo(p: &Point2f) -> geo::Point<f64> {
    geo::Point::new(p.x as f64, p.y as f64)
}

fn from_geo(p: geo::Point<f64>) -> Point2f {
    Point2f::new(p.x() as f32, p.y() as f32)
}

impl Default for CellInstance {
    fn default() -> Self {
        Self {
            cell_definition: Entity::PLACEHOLDER,
            shape_instances: Default::default(),
            child_instances: Default::default(),
            world_transform: Default::default(),
        }
    }
}
