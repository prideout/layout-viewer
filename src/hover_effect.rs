use crate::app_shaders::FRAGMENT_SHADER;
use crate::app_shaders::VERTEX_SHADER;

use crate::graphics::BlendMode;
use crate::graphics::Geometry;
use crate::graphics::Material;
use crate::graphics::Mesh;
use crate::graphics::Ribbon;
use crate::old_core::ElementRef;
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
    pub fn new(world: &mut World) -> Self {
        let mut material = Material::new(VERTEX_SHADER, FRAGMENT_SHADER);
        material.set_blending(BlendMode::SourceOver);
        let fill_material = world.spawn(material).id();

        let geometry = world.spawn(Geometry::new()).id();

        let mut mesh = Mesh::new(geometry, fill_material);
        mesh.visible = false;

        Self {
            polygon: None,
            fill: world.spawn(mesh).id(),
            stroke: Ribbon::new(world),
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

    pub fn set_render_order(&mut self, world: &mut World, render_order: i32) {
        let mut mesh = world.get_mut::<Mesh>(self.fill).unwrap();
        mesh.render_order = render_order;

        self.stroke.set_render_order(world, render_order + 1);
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
