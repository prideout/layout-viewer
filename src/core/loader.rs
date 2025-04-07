use std::collections::BTreeMap;

use crate::graphics::BoundingBox;
use crate::graphics::Geometry;
use crate::graphics::Mesh;

use super::components::CellDefinition;
use super::path_outline::create_path_outline;
use super::path_outline::PathType;
use super::CellReference;
use super::Layer;
use super::LayerMaterial;
use super::LayerMesh;
use super::RootCellInstance;
use super::ShapeDefinition;
use super::ShapeType;
use super::Triangulation;
use super::Vector4f;
use bevy_ecs::entity::Entity;
use bevy_ecs::system::Query;
use bevy_ecs::system::SystemState;
use bevy_ecs::world::World;
use gds21::GdsBoundary;
use gds21::GdsLibrary;
use gds21::GdsPath;
use gds21::GdsPoint;
use gds21::GdsStructRef;

use futures::stream::{self};
use geo::AffineTransform;
use geo::Coord;

type Point2d = nalgebra::Point2<f64>;
type Polygon = geo::Polygon<f64>;
type LineString = geo::LineString<f64>;
type NameTable = BTreeMap<String, Entity>;
type QueryBundle = SystemState<(Query<'static, 'static, (Entity, &'static RootCellInstance)>,)>;

/// Controls the maximum number of GDS elements to consume before yielding.
/// Higher numbers might speed up loading time, but could reduce interactivity
/// and frequency of status update in the UI.
const CHUNK_SIZE: usize = 100;

struct Loader {
    library: Option<GdsLibrary>,
    library_struct_index: usize,
    struct_elem_index: usize,
    world: Option<World>,
    data: Vec<u8>,
    name_to_cell_def: Option<NameTable>,

    // TODO: use this in get_or_create_layer
    queries: QueryBundle,
}

pub struct Progress {
    pub phase: String,
    pub percent: f32,
    pub world: Option<World>,
}

pub async fn load_gds_into_world(
    gds_content: &[u8],
    mut world: World,
) -> impl futures::Stream<Item = Progress> {
    let queries = SystemState::new(&mut world);
    let state = Loader::new(gds_content, world, queries);

    stream::unfold(state, move |mut loader| async move {
        let world = loader.world.as_mut()?;

        loader.queries.get_mut(world); // TODO: use it or lose it

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
                };
                let cell_def = world.spawn(cell_def).id();
                map.insert(gds_struct.name.clone(), cell_def);
            }
            loader.name_to_cell_def = Some(map);
            return loader.next_phase("Creating definitions");
        };

        let gds_struct = &library.structs[loader.library_struct_index];

        for _ in 0..CHUNK_SIZE {
            if loader.struct_elem_index >= gds_struct.elems.len() {
                break;
            }
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
            loader.struct_elem_index += 1;
        }

        if loader.struct_elem_index == gds_struct.elems.len() {
            loader.library_struct_index += 1;
            loader.struct_elem_index = 0;
        }

        let percent = ((loader.library_struct_index as f32) / library.structs.len() as f32) * 100.0;

        // Return ownership of the world only when done loading.
        let world = if percent == 100.0 {
            loader.world.take()
        } else {
            None
        };

        let progress = Progress {
            phase: format!("Creating definitions for '{}'", gds_struct.name),
            percent,
            world,
        };

        Some((progress, loader))
    })
}

impl Loader {
    fn new(gds_content: &[u8], world: World, queries: QueryBundle) -> Self {
        Self {
            library: None,
            library_struct_index: 0,
            struct_elem_index: 0,
            world: Some(world),
            data: gds_content.to_vec(),
            name_to_cell_def: None,
            queries,
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
        let cell_definition = names[&sref.name];

        let translate = AffineTransform::translate(sref.xy.x as f64, sref.xy.y as f64);

        let parent_transform = AffineTransform::identity();

        let mut rotate = AffineTransform::identity();
        let mut scale = AffineTransform::identity();

        if let Some(local_transform) = &sref.strans {
            if let Some(angle) = &local_transform.angle {
                rotate = AffineTransform::rotate(*angle, Coord::zero());
            }
            if local_transform.reflected {
                scale = AffineTransform::scale(1.0, -1.0, Coord::zero());
            }
            if local_transform.mag.unwrap_or(1.0) != 1.0 {
                eprintln!("Magnification not supported.");
            }
            if local_transform.abs_mag || local_transform.abs_angle {
                eprintln!("Absolute transform not supported.");
            }
        }

        let local_transform = scale
            .compose(&rotate)
            .compose(&translate)
            .compose(&parent_transform);

        CellReference {
            cell_definition,
            local_transform,
        }
    }

    // TODO: use passed-in QueryBundle
    fn get_or_create_layer(index: i16, world: &mut World) -> Entity {
        let layer = world
            .query::<(Entity, &Layer)>()
            .iter(world)
            .find(|(_, layer)| layer.index == index);

        if let Some((entity, _)) = layer {
            return entity;
        }

        let layer_material_result = world.query::<(Entity, &LayerMaterial)>().get_single(world);

        let layer_material = match layer_material_result {
            Err(_) => world.spawn(LayerMaterial).id(),
            Ok((entity, _)) => entity,
        };

        let geometry = world.spawn(Geometry::new()).id();

        let mut mesh = Mesh::new(geometry, layer_material);
        mesh.render_order = index as i32;
        let mesh = world.spawn((mesh, LayerMesh)).id();

        let layer = Layer {
            index,
            color: Vector4f::new(0.0, 0.0, 0.0, 1.0),
            visible: true,
            mesh,
            world_bounds: BoundingBox::new(),
            shape_instances: vec![],
        };

        world.spawn(layer).id()
    }

    fn load_boundary(boundary: &GdsBoundary, world: &mut World) -> Entity {
        let geo_points: Vec<_> = boundary.xy.iter().map(gds_to_geo_point).collect();
        let array_points: Vec<_> = boundary.xy.iter().map(gds_point_to_array).collect();
        let local_polygon = Polygon::new(LineString::from(geo_points), vec![]);
        let local_triangles = Triangulation::from_polygon(&local_polygon);
        let layer = Loader::get_or_create_layer(boundary.layer, world);
        let shape_definition = ShapeDefinition {
            layer,
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
        let local_triangles = Triangulation::from_polygon(&local_polygon);
        let layer = Loader::get_or_create_layer(path.layer, world);
        let shape_definition = ShapeDefinition {
            layer,
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
