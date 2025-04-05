use std::collections::BTreeMap;

use super::components::CellDefinition;
use bevy_ecs::{entity::Entity, world::World};
use gds21::GdsLibrary;

use futures::stream::{self};

struct State {
    library: Option<GdsLibrary>,
    index: usize,
    world: Option<World>,
    data: Vec<u8>,
    name_to_cell_def: Option<BTreeMap<String, Entity>>,
}

pub struct Progress {
    pub percent: f32,
    pub world: Option<World>,
}

pub async fn load_gds_into_world(
    gds_content: &[u8],
    world: World,
) -> impl futures::Stream<Item = Progress> {
    let state = State {
        library: None,
        index: 0,
        world: Some(world),
        data: gds_content.to_vec(),
        name_to_cell_def: None,
    };

    stream::unfold(state, move |mut state| async move {
        state.world.as_ref()?;

        let Some(library) = &state.library else {
            let mut data = vec![];
            std::mem::swap(&mut state.data, &mut data);
            let library = GdsLibrary::from_bytes(data).unwrap();
            state.library = Some(library);
            let progress = Progress {
                percent: 0.0,
                world: None,
            };
            return Some((progress, state));
        };

        let Some(name_to_cell_def) = &state.name_to_cell_def else {
            let mut map = BTreeMap::new();
            for gds_struct in &library.structs {
                let cell_def = CellDefinition {
                    name: gds_struct.name.clone(),
                    shape_refs: vec![],
                    cell_refs: vec![],
                    root_instance: None,
                };
                let cell_def = state.world.as_mut().unwrap().spawn(cell_def).id();
                map.insert(gds_struct.name.clone(), cell_def);
            }
            state.name_to_cell_def = Some(map);
            let progress = Progress {
                percent: 0.0,
                world: None,
            };
            return Some((progress, state));
        };

        let gds_struct = &library.structs[state.index];

        let cell_def = name_to_cell_def[&gds_struct.name];
        let mut cell_def = state
            .world
            .as_mut()
            .unwrap()
            .get_mut::<CellDefinition>(cell_def)
            .unwrap();

        for elem in &gds_struct.elems {
            match elem {
                gds21::GdsElement::GdsStructRef(sref) => {
                    let name = &sref.name;
                    let xy = &sref.xy;
                    let strans = &sref.strans;
                }
                gds21::GdsElement::GdsArrayRef(_) => {
                    // TODO: array refs are not yet implemented, hide them for now
                }
                gds21::GdsElement::GdsBoundary(boundary) => {
                    // TODO
                }
                gds21::GdsElement::GdsPath(path) => {
                    // TODO
                }
                gds21::GdsElement::GdsTextElem(_) => {
                    // We do not support text elements yet, but they do
                    // occur so let's not spam the console with warnings.
                }
                gds21::GdsElement::GdsNode(_) => {
                    log::warn!("Node elements are not supported");
                }
                gds21::GdsElement::GdsBox(_) => {
                    log::warn!("Box elements are not supported");
                }
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        }

        #[cfg(target_arch = "wasm32")]
        {
            gloo_timers::future::TimeoutFuture::new(50).await;
        }

        let percent = ((1.0 + state.index as f32) / library.structs.len() as f32) * 100.0;

        state.index += 1;

        // Return ownership of the world only when done loading.
        let world = if percent == 100.0 {
            state.world.take()
        } else {
            None
        };

        let progress = Progress { percent, world };

        Some((progress, state))
    })
}
