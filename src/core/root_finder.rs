use std::collections::HashSet;

use bevy_ecs::entity::Entity;
use bevy_ecs::query::With;
use bevy_ecs::system::Query;
use bevy_ecs::system::SystemState;
use bevy_ecs::world::EntityRef;
use bevy_ecs::world::World;

use super::components::CellDefinition;

/// Finds CellDefinition entities that are not referenced by any other CellDefinition.
pub struct RootFinder {
    query: SystemState<Query<'static, 'static, EntityRef<'static>, With<CellDefinition>>>,
    visited: HashSet<Entity>,
    non_roots: HashSet<Entity>,
}

impl RootFinder {
    pub fn new(world: &mut World) -> Self {
        Self {
            query: SystemState::new(world),
            visited: HashSet::new(),
            non_roots: HashSet::new(),
        }
    }

    /// Finds CellDefinition entities that are not referenced by any other CellDefinition.
    pub fn find_roots(&mut self, world: &World) -> Vec<Entity> {
        self.visited.clear();
        self.non_roots.clear();

        let query = self.query.get(world);

        for cell in query.iter() {
            if self.visited.contains(&cell.id()) {
                continue;
            }
            self.visited.insert(cell.id());
            let cell_def = cell.get::<CellDefinition>().unwrap();
            for cell_ref in &cell_def.cell_refs {
                self.non_roots.insert(cell_ref.cell_definition);
            }
        }

        let mut roots = Vec::new();
        for cell in query.iter() {
            if !self.non_roots.contains(&cell.id()) {
                roots.push(cell.id());
            }
        }
        roots
    }
}
