use bevy_ecs::entity::Entity;
use bevy_ecs::query::QueryState;
use bevy_ecs::system::lifetimeless::Read;
use bevy_ecs::world::World;
use geo::AffineOps;
use geo::AffineTransform;
use geo::BoundingRect;

use crate::core::components::CellDefinition;
use crate::core::components::CellInstance;
use crate::core::components::Layer;
use crate::core::components::RootCellInstance;
use crate::core::components::ShapeDefinition;
use crate::core::components::ShapeInstance;
use crate::core::triangulation::Triangulation;
use crate::graphics::bounds::BoundingBox;
use crate::graphics::geometry::Geometry;
use crate::graphics::mesh::Mesh;
use crate::graphics::vectors::*;

pub struct Instancer {
    root_query: QueryState<(Entity, Read<RootCellInstance>)>,
}

impl Instancer {
    pub fn new(world: &mut World) -> Self {
        Self {
            root_query: world.query(),
        }
    }

    /// Selects a cell definition as the root of the instance tree, then
    /// instantiates the entire tree of CellInstance entities.
    pub fn select_root(&mut self, world: &mut World, cell_definition_id: Entity) {
        let Some(cell_definition) = world.get::<CellDefinition>(cell_definition_id) else {
            panic!("Entity does not have a CellDefinition component");
        };

        let existing_root = self.root_query.get_single(world);
        if existing_root.is_ok() {
            panic!("Root cell instance already exists");
        }

        log::info!("Selecting {} as root.", cell_definition.name);

        let identity = AffineTransform::identity();
        let root = Instancer::instantiate(world, cell_definition_id, identity);
        world.get_entity_mut(root).unwrap().insert(RootCellInstance);
    }

    /// Recursively creates cell instances and returns the instance corresponding
    /// to the given cell_definition_id.
    fn instantiate(
        world: &mut World,
        cell_definition_id: Entity,
        transform: AffineTransform,
    ) -> Entity {
        let Some(cell_definition) = world.get::<CellDefinition>(cell_definition_id) else {
            panic!("Entity does not have a CellDefinition component");
        };

        // Phase 1: Gathering (immutable access to world)

        struct ShapePrototype {
            layer: Entity,
            world_polygon: Polygon,
            world_triangles: Triangulation,
        }

        let mut shape_prototypes = Vec::new();
        for shape_def in &cell_definition.shape_defs {
            let shape_def = world.get::<ShapeDefinition>(*shape_def);
            let Some(shape_def) = shape_def else {
                log::error!("Shape definition not found");
                continue;
            };
            let layer = shape_def.layer;
            let world_polygon = shape_def.local_polygon.affine_transform(&transform);
            let world_triangles = shape_def.local_triangles.affine_transform(&transform);
            shape_prototypes.push(ShapePrototype {
                layer,
                world_polygon,
                world_triangles,
            });
        }

        let shape_prototypes = shape_prototypes;
        let cell_prototypes = cell_definition.cell_refs.clone();

        // Phase 2: Production (mutable access to world)

        let cell_instance_id = world.spawn_empty().id();
        let parent_transform = transform;

        let mut shape_instances = Vec::with_capacity(shape_prototypes.len());
        for prototype in shape_prototypes {
            let layer = world.get_mut::<Layer>(prototype.layer).unwrap();
            let layer_index = layer.index;
            let mesh = layer.mesh;
            let bbox = prototype.world_polygon.bounding_rect();
            let shape_instance = ShapeInstance {
                cell_instance: cell_instance_id,
                world_polygon: prototype.world_polygon,
                layer_index,
                layer: prototype.layer,
            };
            let shape_instance_id = world.spawn(shape_instance).id();
            shape_instances.push(shape_instance_id);
            let mut layer = world.get_mut::<Layer>(prototype.layer).unwrap();
            layer.shape_instances.push(shape_instance_id);
            if let Some(bbox) = bbox {
                let bbox = BoundingBox::from(bbox);
                layer.world_bounds.encompass(&bbox);
            }
            let geo = world.get::<Mesh>(mesh).unwrap().geometry;
            let mut geo = world.get_mut::<Geometry>(geo).unwrap();
            prototype.world_triangles.append_to(&mut geo);
        }

        let mut child_instances = Vec::with_capacity(cell_prototypes.len());
        for cell_ref in cell_prototypes {
            let transform = cell_ref.local_transform.compose(&parent_transform);
            let child_definition = cell_ref.cell_definition;
            let child = Instancer::instantiate(world, child_definition, transform);
            child_instances.push(child);
        }

        let cell_instance = CellInstance {
            cell_definition: cell_definition_id,
            shape_instances,
            child_instances,
            world_transform: parent_transform,
        };

        world
            .get_entity_mut(cell_instance_id)
            .unwrap()
            .insert(cell_instance);

        cell_instance_id
    }
}
