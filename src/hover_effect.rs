use crate::app_shaders::FRAGMENT_SHADER;
use crate::app_shaders::VERTEX_SHADER;

use crate::core::ElementRef;
use crate::graphics::BlendMode;
use crate::graphics::Geometry;
use crate::graphics::Material;
use crate::graphics::Mesh;
use crate::graphics::Ribbon;
use crate::graphics::Scene;
use crate::Project;

use bevy_ecs::entity::Entity;
use bevy_ecs::world::World;
use geo::TriangulateEarcut;

type Point2 = nalgebra::Point2<f64>;

/// Parameters for setting a cell in the hover effect
pub struct HoverParams<'a> {
    pub selection: ElementRef,
    pub project: &'a Project,
    pub world: &'a mut World,
    pub gl: &'a glow::Context,
}

/// Manages graphics primitives for a hover effect
pub struct HoverEffect {
    polygon: Option<ElementRef>,
    fill: Entity,
    stroke: Ribbon,
}

impl HoverEffect {
    pub fn new(world: &mut World, scene: &mut Scene) -> Self {
        let mut fill_material = Material::new(VERTEX_SHADER, FRAGMENT_SHADER);
        fill_material.set_blending(BlendMode::SourceOver);
        let fill_material = scene.add_material(fill_material);

        let fill_geometry = world.spawn_empty().id();
        world.entity_mut(fill_geometry).insert(Geometry::new());

        let mut fill_mesh = Mesh::new(fill_geometry, fill_material);
        fill_mesh.visible = false;

        let fill = world.spawn_empty().id();
        world.entity_mut(fill).insert(fill_mesh);
        let ribbon = Ribbon::new(world, scene);

        Self {
            polygon: None,
            fill,
            stroke: ribbon,
        }
    }

    pub fn update_stroke_width(&mut self, width: f64, world: &mut World, gl: &glow::Context) {
        if self.stroke.width != width {
            self.stroke.width = width;
            self.stroke.update(world, gl);
        }
    }

    pub fn contains(&self, polygon: &ElementRef) -> bool {
        self.polygon == Some(polygon.clone())
    }

    pub fn move_to_back(&mut self) {
        // scene.move_mesh_to_back(self.fill);
        // scene.move_mesh_to_back(self.stroke.mesh());
    }

    pub fn is_visible(&self) -> bool {
        self.polygon.is_some()
    }

    pub fn hide(&mut self, world: &mut World) {
        self.polygon = None;
        let mut mesh = world.get_mut::<Mesh>(self.fill).unwrap();
        mesh.visible = false;
        self.stroke.hide(world);
    }

    /// Activates the hover effect for a specific polygon.
    pub fn show(
        &mut self,
        HoverParams {
            selection,
            project,
            world,
            gl,
        }: HoverParams,
    ) {
        self.polygon = Some(selection.clone());

        let layer = &project.layers()[selection.layer as usize];
        let polygon = &layer.element_instances[selection.element_instance_index].polygon;

        let triangles = polygon.earcut_triangles_raw();

        let mut color = layer.color;
        color.w = 0.5;

        let mut points = Vec::new();
        for coord in polygon.exterior().points() {
            points.push(Point2::new(coord.x(), coord.y()));
        }

        self.stroke.spine = points;
        self.stroke.update(world, gl);

        let mut geometry = Geometry::new();

        geometry.positions.reserve(3 * triangles.vertices.len() / 2);
        geometry.indices.reserve(triangles.triangle_indices.len());

        for coord in triangles.vertices.chunks(2) {
            geometry.positions.push(coord[0] as f32);
            geometry.positions.push(coord[1] as f32);
            geometry.positions.push(0.0);
        }

        for index in triangles.triangle_indices {
            geometry.indices.push(index as u32);
        }

        let mut mesh = world.get_mut::<Mesh>(self.fill).unwrap();
        mesh.visible = true;
        mesh.set_vec4("color", color);
        let geometry_entity = mesh.geometry;
        geometry.replace(world, gl, geometry_entity);
    }
}
