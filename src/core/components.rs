use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use geo::AffineTransform;

use crate::core::triangulation::Triangulation;
use crate::graphics::bounds::BoundingBox;
use crate::graphics::material::Material;
use crate::graphics::mesh::Mesh;
use crate::graphics::vectors::*;

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
#[require(Mesh)]
pub struct LayerMesh;

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

/// This component is referenced by the R-tree that we use for fast spatial
/// lookups. Each node in the tree has:
/// - this entity id
/// - the aabb of the world_polygon
/// - a copy of the layer index
#[derive(Component)]
pub struct ShapeInstance {
    pub cell_instance: Entity,
    pub world_polygon: Polygon,
    pub layer_index: i16,
    pub layer: Entity,
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
