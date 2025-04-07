use std::collections::HashSet;

use bevy_ecs::entity::Entity;
use bevy_ecs::query::QueryState;
use bevy_ecs::world::World;

use super::components::CellDefinition;

/// Finds CellDefinition entities that are not referenced by any other CellDefinition.
pub struct RootFinder<'world> {
    query: QueryState<(Entity, &'world CellDefinition)>,
    visited: HashSet<Entity>,
    non_roots: HashSet<Entity>,
}

impl RootFinder<'_> {
    pub fn new(world: &mut World) -> Self {
        Self {
            query: world.query::<(Entity, &CellDefinition)>(),
            visited: HashSet::new(),
            non_roots: HashSet::new(),
        }
    }

    /// Finds CellDefinition entities that are not referenced by any other CellDefinition.
    pub fn find_roots(&mut self, world: &World) -> Vec<Entity> {
        self.visited.clear();
        self.non_roots.clear();

        for (entity, cell) in self.query.iter(world) {
            if self.visited.contains(&entity) {
                continue;
            }
            self.visited.insert(entity);
            for cell_ref in &cell.cell_refs {
                self.non_roots.insert(cell_ref.cell_definition);
            }
        }

        let mut roots = Vec::new();
        for (entity, _) in self.query.iter(world) {
            if !self.non_roots.contains(&entity) {
                roots.push(entity);
            }
        }
        roots
    }
}
