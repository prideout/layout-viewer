use crate::core::components::CellDefinition;
use crate::core::components::CellReference;
use crate::core::components::Layer;
use crate::core::components::LayerMaterial;
use crate::core::components::LayerMesh;
use crate::core::components::ShapeDefinition;
use crate::core::components::ShapeType;
use crate::core::path_outline::create_path_outline;
use crate::core::path_outline::PathType;
use crate::core::triangulation::Triangulation;
use crate::graphics::bounds::BoundingBox;
use crate::graphics::geometry::Geometry;
use crate::graphics::mesh::Mesh;
use crate::graphics::vectors::*;
use crate::rsutils::moonshine::SpawnInstanceWorld;
use std::collections::BTreeMap;

use bevy_ecs::entity::Entity;
use bevy_ecs::query::QueryState;
use bevy_ecs::world::World;
use gds21::GdsBoundary;
use gds21::GdsLibrary;
use gds21::GdsPath;
use gds21::GdsPoint;
use gds21::GdsStructRef;
use geo::AffineTransform;
use geo::Coord;
use geo::LineString;
use moonshine_kind::Instance;
use web_time::Instant;

type NameTable = BTreeMap<String, Entity>;

const BUDGET_MS: u128 = 15;

pub struct Progress {
    phase: String,
    percent: f32,
    world: Option<World>,
}

/// Reads a GDS file, creates a World, and populates it with definition
/// entities.
///
/// Has an iterator interface to allow progress reporting and
/// periodic yielding to the UI.
///
/// Does not create instance entities; for that see `Instancer`.
pub struct Loader {
    state: Option<LoaderState>,
}

impl Loader {
    pub fn new(gds_content: &[u8]) -> Self {
        let state = LoaderState::ParsingFile(gds_content.to_vec());
        Self { state: Some(state) }
    }
}

impl Iterator for Loader {
    type Item = Progress;

    fn next(&mut self) -> Option<Progress> {
        let state = self.state.take()?;
        let (progress, state) = state.next()?;
        self.state = Some(state);
        Some(progress)
    }
}

impl Progress {
    pub fn status_message(&self) -> String {
        if self.percent > 0.0 {
            format!("{} {:.0}%", self.phase, self.percent)
        } else {
            self.phase.clone()
        }
    }

    pub fn take_world(&mut self) -> Option<World> {
        self.world.take()
    }
}

enum LoaderState {
    ParsingFile(Vec<u8>),
    GatheringNames(GdsLibrary),
    GeneratingWorld(Box<WorldGenerator>),
    YieldingWorld(Box<World>),
    Done,
}

impl LoaderState {
    fn next(self) -> Option<(Progress, Self)> {
        match self {
            LoaderState::ParsingFile(data) => {
                let library = GdsLibrary::from_bytes(data).unwrap();
                next_state("Parsing file", LoaderState::GatheringNames(library))
            }
            LoaderState::GatheringNames(library) => {
                let mut world = World::new();
                let mut map = BTreeMap::new();
                let mut count = 0;
                for gds_struct in &library.structs {
                    let cell_def = CellDefinition {
                        name: gds_struct.name.clone(),
                        shape_defs: vec![],
                        cell_refs: vec![],
                    };
                    let cell_def = world.spawn(cell_def).id();
                    map.insert(gds_struct.name.clone(), cell_def);
                    count += gds_struct.elems.len();
                }
                let generator = WorldGenerator::new(world, library, map, count);
                next_state("Generating world", LoaderState::GeneratingWorld(generator))
            }
            LoaderState::GeneratingWorld(mut generator) => {
                let start = Instant::now();
                for _ in 0..generator.chunk_size {
                    generator.process_element();
                    if generator.is_done() {
                        let world = Box::new(generator.world);
                        return next_state("Done", LoaderState::YieldingWorld(world));
                    }
                }
                let end = Instant::now();
                let duration = end - start;
                if duration.as_millis() > BUDGET_MS {
                    generator.chunk_size = std::cmp::max(generator.chunk_size - 1, 1);
                } else {
                    generator.chunk_size += 1;
                }
                let progress = generator.progress();
                Some((progress, LoaderState::GeneratingWorld(generator)))
            }
            LoaderState::YieldingWorld(world) => {
                // Move the world from LoaderState to Progress so that the
                // caller can take ownership of it.
                let progress = Progress {
                    phase: "Done".to_string(),
                    percent: 100.0,
                    world: Some(*world),
                };
                Some((progress, LoaderState::Done))
            }
            LoaderState::Done => None,
        }
    }
}

struct WorldGenerator {
    world: World,
    library: GdsLibrary,
    name_to_cell_def: NameTable,
    struct_index: usize,
    element_index: usize,
    total_element_count: usize,
    processed_element_count: usize,
    status: String,
    layer_query: QueryState<(Entity, &'static Layer)>,
    layer_material_query: QueryState<(Entity, &'static LayerMaterial)>,

    /// Controls the maximum number of GDS elements to process before yielding.
    /// Higher numbers might speed up loading time, but could reduce interactivity
    /// and frequency of status updates in the UI.
    chunk_size: usize,
}

impl WorldGenerator {
    fn new(
        mut world: World,
        library: GdsLibrary,
        name_to_cell_def: NameTable,
        total_element_count: usize,
    ) -> Box<Self> {
        let layer_query = QueryState::new(&mut world);
        let layer_material_query = QueryState::new(&mut world);

        Box::new(WorldGenerator {
            world,
            library,
            name_to_cell_def,
            struct_index: 0,
            element_index: 0,
            total_element_count,
            processed_element_count: 0,
            status: String::new(),
            layer_query,
            layer_material_query,
            chunk_size: 300,
        })
    }

    fn progress(&self) -> Progress {
        Progress {
            phase: self.status.clone(),
            percent: self.fraction() * 100.0,
            world: None,
        }
    }

    fn is_done(&self) -> bool {
        self.processed_element_count >= self.total_element_count
    }

    fn fraction(&self) -> f32 {
        (self.processed_element_count as f32) / (self.total_element_count as f32)
    }

    fn process_element(&mut self) {
        let gds_struct = &self.library.structs[self.struct_index];
        if self.element_index >= gds_struct.elems.len() {
            self.struct_index += 1;
            self.element_index = 0;
            if self.struct_index >= self.library.structs.len() {
                return;
            }
        }
        let gds_struct = &self.library.structs[self.struct_index];
        let cell_def = self.name_to_cell_def[&gds_struct.name];
        self.status = gds_struct.name.clone();
        let element = &gds_struct.elems[self.element_index];
        match element {
            gds21::GdsElement::GdsStructRef(sref) => {
                let cell_ref = self.load_struct_ref(&sref.clone());
                let mut cell_def = self.world.get_mut::<CellDefinition>(cell_def).unwrap();
                cell_def.cell_refs.push(cell_ref);
            }
            gds21::GdsElement::GdsArrayRef(_) => {
                // TODO: array refs are not yet implemented, hide them for now
            }
            gds21::GdsElement::GdsBoundary(boundary) => {
                let shape_def = self.load_boundary(&boundary.clone());
                let mut cell_def = self.world.get_mut::<CellDefinition>(cell_def).unwrap();
                cell_def.shape_defs.push(shape_def);
            }
            gds21::GdsElement::GdsPath(path) => {
                let shape_def = self.load_path(&path.clone());
                let mut cell_def = self.world.get_mut::<CellDefinition>(cell_def).unwrap();
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
        self.element_index += 1;
        self.processed_element_count += 1;
    }

    fn load_struct_ref(&mut self, sref: &GdsStructRef) -> CellReference {
        let cell_definition = self.name_to_cell_def[&sref.name];

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

    fn load_boundary(&mut self, boundary: &GdsBoundary) -> Instance<ShapeDefinition> {
        let geo_points: Vec<_> = boundary.xy.iter().map(gds_to_geo_point).collect();
        let array_points: Vec<_> = boundary.xy.iter().map(gds_point_to_array).collect();
        let local_polygon = Polygon::new(LineString::from(geo_points), vec![]);
        let local_triangles = Triangulation::from_polygon(&local_polygon);
        let layer = self.get_or_create_layer(boundary.layer);
        let shape_definition = ShapeDefinition {
            layer,
            shape_type: ShapeType::Polygon(array_points),
            local_polygon,
            local_triangles,
        };
        self.world.spawn_instance_ref(shape_definition).instance()
    }

    fn load_path(&mut self, path: &GdsPath) -> Instance<ShapeDefinition> {
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
        let layer = self.get_or_create_layer(path.layer);
        let shape_definition = ShapeDefinition {
            layer,
            shape_type: ShapeType::Path { width, spine },
            local_polygon,
            local_triangles,
        };
        self.world.spawn_instance_id(shape_definition)
    }

    fn get_or_create_layer(&mut self, index: i16) -> Entity {
        let layer = self
            .layer_query
            .iter(&self.world)
            .find(|(_, layer)| layer.index == index);

        if let Some((entity, _)) = layer {
            return entity;
        }

        let layer_material_result = self.layer_material_query.get_single(&self.world);

        let layer_material = match layer_material_result {
            Err(_) => self.world.spawn(LayerMaterial).id(),
            Ok((entity, _)) => entity,
        };

        let geometry = self.world.spawn(Geometry::new()).id();

        let mut mesh = Mesh::new(geometry, layer_material);
        mesh.render_order = index as i32;
        let mesh = self.world.spawn((mesh, LayerMesh)).id();

        let layer = Layer {
            index,
            color: Vector4f::new(0.0, 0.0, 0.0, 1.0),
            visible: true,
            mesh,
            world_bounds: BoundingBox::new(),
            shape_instances: vec![],
        };

        self.world.spawn(layer).id()
    }
}

fn gds_to_geo_point(p: &GdsPoint) -> geo::Point<f64> {
    geo::Point::<f64>::new(p.x as f64, p.y as f64)
}

fn gds_point_to_array(p: &GdsPoint) -> Point2d {
    Point2d::new(p.x as f64, p.y as f64)
}

fn next_state(phase: &str, state: LoaderState) -> Option<(Progress, LoaderState)> {
    let progress = Progress {
        phase: phase.to_string(),
        percent: 0.0,
        world: None,
    };
    Some((progress, state))
}
