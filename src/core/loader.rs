use std::collections::BTreeMap;

use crate::core::components::CellDefinition;
use crate::core::components::CellReference;
use crate::core::components::Layer;
use crate::core::components::LayerMaterial;
use crate::core::components::LayerMesh;
use crate::core::components::RootCellInstance;
use crate::core::components::ShapeDefinition;
use crate::core::components::ShapeType;
use crate::core::path_outline::create_path_outline;
use crate::core::path_outline::PathType;
use crate::core::triangulation::Triangulation;
use crate::graphics::bounds::BoundingBox;
use crate::graphics::geometry::Geometry;
use crate::graphics::mesh::Mesh;
use crate::graphics::vectors::*;

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

type LineString = geo::LineString<f64>;
type NameTable = BTreeMap<String, Entity>;
type QueryBundle = SystemState<(Query<'static, 'static, (Entity, &'static RootCellInstance)>,)>;

/// Controls the maximum number of GDS elements to process before yielding.
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
    #[allow(dead_code)]
    queries: QueryBundle,

    processed_element_count: usize,
    total_element_count: usize,
    status: String,
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
        loader.world.as_mut()?;

        if loader.library.is_none() {
            let mut data = vec![];
            std::mem::swap(&mut loader.data, &mut data);
            let library = GdsLibrary::from_bytes(data).unwrap();
            loader.library = Some(library);
            return loader.next_phase("Gathering definitions");
        };

        if loader.name_to_cell_def.is_none() {
            let world = loader.world.as_mut()?;
            let library = loader.library.as_ref().unwrap();
            let mut map = BTreeMap::new();
            loader.total_element_count = 0;
            for gds_struct in &library.structs {
                let cell_def = CellDefinition {
                    name: gds_struct.name.clone(),
                    shape_defs: vec![],
                    cell_refs: vec![],
                };
                let cell_def = world.spawn(cell_def).id();
                map.insert(gds_struct.name.clone(), cell_def);
                loader.total_element_count += gds_struct.elems.len();
            }
            loader.name_to_cell_def = Some(map);
            return loader.next_phase("Creating definitions");
        };

        let mut fraction = 0.0;
        for _ in 0..CHUNK_SIZE {
            loader.process_element();
            fraction = (loader.processed_element_count as f32) / loader.total_element_count as f32;
            if fraction == 1.0 {
                break;
            }
        }

        // Return ownership of the world only when done loading.
        let world = if fraction == 1.0 {
            loader.world.take()
        } else {
            None
        };

        let progress = Progress {
            phase: format!("Creating definitions for '{}'", loader.status),
            percent: fraction * 100.0,
            world,
        };

        Some((progress, loader))
    })
}

impl Loader {
    fn new(gds_content: &[u8], mut world: World) -> Self {
        let queries = SystemState::new(&mut world);
        Self {
            library: None,
            library_struct_index: 0,
            struct_elem_index: 0,
            world: Some(world),
            data: gds_content.to_vec(),
            name_to_cell_def: None,
            queries,
            total_element_count: 0,
            processed_element_count: 0,
            status: String::new(),
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

    fn process_element(&mut self) {
        let library = self.library.as_ref().unwrap();
        let world = self.world.as_mut().unwrap();
        let gds_struct = &library.structs[self.library_struct_index];
        let name_to_cell_def = self.name_to_cell_def.as_ref().unwrap();
        if self.struct_elem_index >= gds_struct.elems.len() {
            self.library_struct_index += 1;
            self.struct_elem_index = 0;
            if self.library_struct_index >= library.structs.len() {
                return;
            }
        }
        let gds_struct = &library.structs[self.library_struct_index];
        let element = &gds_struct.elems[self.struct_elem_index];
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
        self.struct_elem_index += 1;
        self.processed_element_count += 1;
        self.status = gds_struct.name.clone();
    }
}

fn gds_to_geo_point(p: &GdsPoint) -> geo::Point<f64> {
    geo::Point::<f64>::new(p.x as f64, p.y as f64)
}

fn gds_point_to_array(p: &GdsPoint) -> Point2d {
    Point2d::new(p.x as f64, p.y as f64)
}
