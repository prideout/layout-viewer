use std::collections::BTreeMap;

use super::components::CellDefinition;
use super::path_outline::create_path_outline;
use super::path_outline::PathType;
use super::CellReference;
use super::ShapeDefinition;
use super::ShapeType;
use super::Triangulation;
use bevy_ecs::entity::Entity;
use bevy_ecs::world::World;
use gds21::GdsBoundary;
use gds21::GdsLibrary;
use gds21::GdsPath;
use gds21::GdsPoint;
use gds21::GdsStructRef;

use futures::stream::{self};
use geo::AffineTransform;
use geo::TriangulateEarcut;

type Point2d = nalgebra::Point2<f64>;
type Point2f = nalgebra::Point2<f32>;
type Polygon = geo::Polygon<f64>;
type LineString = geo::LineString<f64>;
type NameTable = BTreeMap<String, Entity>;

struct Loader {
    library: Option<GdsLibrary>,
    library_struct_index: usize,
    struct_elem_index: usize,
    world: Option<World>,
    data: Vec<u8>,
    name_to_cell_def: Option<NameTable>,
}

pub struct Progress {
    pub phase: String,
    pub percent: f32,
    pub world: Option<World>,
}

pub async fn load_gds_into_world(
    gds_content: &[u8],
    world: World,
) -> impl futures::Stream<Item = Progress> {
    let state = Loader::new(gds_content, world);

    stream::unfold(state, move |mut loader| async move {
        let world = loader.world.as_mut()?;

        let Some(library) = &loader.library else {
            let mut data = vec![];
            std::mem::swap(&mut loader.data, &mut data);
            let library = GdsLibrary::from_bytes(data).unwrap();
            loader.library = Some(library);
            return loader.next_phase("Gathering definitions");
        };

        let Some(name_to_cell_def) = &loader.name_to_cell_def else {
            let mut map = BTreeMap::new();
            for gds_struct in &library.structs {
                let cell_def = CellDefinition {
                    name: gds_struct.name.clone(),
                    shape_defs: vec![],
                    cell_refs: vec![],
                    root_instance: None,
                };
                let cell_def = world.spawn(cell_def).id();
                map.insert(gds_struct.name.clone(), cell_def);
            }
            loader.name_to_cell_def = Some(map);
            return loader.next_phase("Creating definitions");
        };

        let gds_struct = &library.structs[loader.library_struct_index];
        let element = &gds_struct.elems[loader.struct_elem_index];
        match element {
            gds21::GdsElement::GdsStructRef(sref) => {
                let cell_ref = Loader::load_struct_ref(sref, name_to_cell_def);
                let cell_def = name_to_cell_def[&gds_struct.name];
                let mut cell_def = world.get_mut::<CellDefinition>(cell_def).unwrap();
                cell_def.cell_refs.push(cell_ref);
            }
            gds21::GdsElement::GdsArrayRef(_) => {
                // TODO: array refs are not yet implemented, hide them for now
            }
            gds21::GdsElement::GdsBoundary(boundary) => {
                let shape_def = Loader::load_boundary(boundary, world);
                let cell_def = name_to_cell_def[&gds_struct.name];
                let mut cell_def = world.get_mut::<CellDefinition>(cell_def).unwrap();
                cell_def.shape_defs.push(shape_def);
            }
            gds21::GdsElement::GdsPath(path) => {
                let shape_def = Loader::load_path(path, world);
                let cell_def = name_to_cell_def[&gds_struct.name];
                let mut cell_def = world.get_mut::<CellDefinition>(cell_def).unwrap();
                cell_def.shape_defs.push(shape_def);
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

        if loader.struct_elem_index == gds_struct.elems.len() - 1 {
            #[cfg(not(target_arch = "wasm32"))]
            {
                tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
            }

            #[cfg(target_arch = "wasm32")]
            {
                gloo_timers::future::TimeoutFuture::new(50).await;
            }

            loader.library_struct_index += 1;
            loader.struct_elem_index = 0;
        } else {
            loader.struct_elem_index += 1;
        }

        let percent = ((loader.library_struct_index as f32) / library.structs.len() as f32) * 100.0;

        // Return ownership of the world only when done loading.
        let world = if percent == 100.0 {
            loader.world.take()
        } else {
            None
        };

        let progress = Progress {
            phase: "Creating definitions".to_string(),
            percent,
            world,
        };

        Some((progress, loader))
    })
}

impl Loader {
    fn new(gds_content: &[u8], world: World) -> Self {
        Self {
            library: None,
            library_struct_index: 0,
            struct_elem_index: 0,
            world: Some(world),
            data: gds_content.to_vec(),
            name_to_cell_def: None,
        }
    }

    fn next_phase(self, phase: &str) -> Option<(Progress, Self)> {
        let progress = Progress {
            phase: phase.to_string(),
            percent: 0.0,
            world: None,
        };
        Some((progress, self))
    }

    fn load_struct_ref(sref: &GdsStructRef, names: &NameTable) -> CellReference {
        let cell_def = names[&sref.name];
        CellReference {
            cell_definition: cell_def,
            local_transform: AffineTransform::identity(),
        }
    }

    fn triangulate_polygon(polygon: &Polygon) -> Triangulation {
        let earcut_result = polygon.earcut_triangles_raw();
        let mut vertices = Vec::with_capacity(earcut_result.vertices.len() / 2);
        for coord in earcut_result.vertices.chunks(2) {
            vertices.push(Point2f::new(coord[0] as f32, coord[1] as f32));
        }
        let mut indices = Vec::with_capacity(earcut_result.triangle_indices.len());
        for i in earcut_result.triangle_indices {
            indices.push(i as u32);
        }
        Triangulation { indices, vertices }
    }

    fn load_boundary(boundary: &GdsBoundary, world: &mut World) -> Entity {
        let geo_points: Vec<_> = boundary.xy.iter().map(gds_to_geo_point).collect();
        let array_points: Vec<_> = boundary.xy.iter().map(gds_point_to_array).collect();
        let local_polygon = Polygon::new(LineString::from(geo_points), vec![]);
        let local_triangles = Loader::triangulate_polygon(&local_polygon);
        let shape_definition = ShapeDefinition {
            layer: Entity::PLACEHOLDER,
            shape_type: ShapeType::Polygon(array_points),
            local_polygon,
            local_triangles,
        };
        world.spawn(shape_definition).id()
    }

    fn load_path(path: &GdsPath, world: &mut World) -> Entity {
        let spine: Vec<_> = path.xy.iter().map(gds_point_to_array).collect();
        let width = path.width.unwrap_or(0) as f64;
        let half_width = width / 2.0;

        let path_type = path
            .path_type
            .map(PathType::from)
            .unwrap_or(PathType::Standard);

        let outline_points = create_path_outline(&path.xy, half_width, path_type);
        let local_polygon = Polygon::new(LineString::from(outline_points), vec![]);
        let local_triangles = Loader::triangulate_polygon(&local_polygon);
        let shape_definition = ShapeDefinition {
            layer: Entity::PLACEHOLDER,
            shape_type: ShapeType::Path { width, spine },
            local_polygon,
            local_triangles,
        };
        world.spawn(shape_definition).id()
    }
}

fn gds_to_geo_point(p: &GdsPoint) -> geo::Point<f64> {
    geo::Point::<f64>::new(p.x as f64, p.y as f64)
}

fn gds_point_to_array(p: &GdsPoint) -> Point2d {
    Point2d::new(p.x as f64, p.y as f64)
}
