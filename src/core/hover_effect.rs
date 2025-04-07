use crate::core::components::Layer;
use crate::core::components::LayerMaterial;
use crate::core::components::ShapeInstance;
use crate::graphics::geometry::Geometry;
use crate::graphics::material::BlendMode;
use crate::graphics::material::Material;
use crate::graphics::mesh::Mesh;
use crate::graphics::ribbon::Ribbon;
use crate::graphics::vectors::*;
use bevy_ecs::entity::Entity;
use bevy_ecs::world::World;
use geo::TriangulateEarcut;

/// Parameters for setting a cell in the hover effect
pub struct HoverParams<'a> {
    pub shape_instance: Entity,
    pub world: &'a mut World,
    pub gl: &'a glow::Context,
}

/// Manages graphics primitives for a hover effect
pub struct HoverEffect {
    fill_mesh: Entity,
    stroke: Ribbon,
}

impl HoverEffect {
    pub fn new(world: &mut World) -> Self {
        let mut lmq = world.query::<(Entity, &LayerMaterial, &mut Material)>();
        let fill_material = lmq.get_single_mut(world).ok().map(|(entity, _, _)| entity);

        let fill_material = fill_material.unwrap_or_else(|| {
            let mut material = Material::default();
            material.set_blending(BlendMode::SourceOver);
            world.spawn(material).id()
        });

        let geometry = world.spawn(Geometry::new()).id();

        let mut mesh = Mesh::new(geometry, fill_material);
        mesh.visible = false;

        Self {
            fill_mesh: world.spawn(mesh).id(),
            stroke: Ribbon::new(world),
        }
    }

    pub fn update_stroke_width(&mut self, width: f64, world: &mut World, gl: &glow::Context) {
        if self.stroke.width != width {
            self.stroke.width = width;
            self.stroke.update(world, gl);
        }
    }

    pub fn set_render_order(&mut self, world: &mut World, render_order: i32) {
        let mut mesh = world.get_mut::<Mesh>(self.fill_mesh).unwrap();
        mesh.render_order = render_order;

        self.stroke.set_render_order(world, render_order + 1);
    }

    pub fn hide(&mut self, world: &mut World) {
        let mut mesh = world.get_mut::<Mesh>(self.fill_mesh).unwrap();
        mesh.visible = false;
        self.stroke.hide(world);
    }

    /// Activates the hover effect for a specific polygon.
    pub fn show(
        &mut self,
        HoverParams {
            shape_instance,
            world,
            gl,
        }: HoverParams,
    ) {
        let shape_instance = world.get::<ShapeInstance>(shape_instance).unwrap();
        let triangles = shape_instance.world_polygon.earcut_triangles_raw();

        let mut points = Vec::new();
        for coord in shape_instance.world_polygon.exterior().points() {
            points.push(Point2d::new(coord.x(), coord.y()));
        }

        let layer = world.get::<Layer>(shape_instance.layer).unwrap();
        let mut color = layer.color;
        color.w *= 0.1;

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

        let mut mesh = world.get_mut::<Mesh>(self.fill_mesh).unwrap();
        mesh.visible = true;
        mesh.set_vec4("color", color);
        let geometry_entity = mesh.geometry;
        geometry.replace(world, gl, geometry_entity);
    }
}
