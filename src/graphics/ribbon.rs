use bevy_ecs::entity::Entity;
use bevy_ecs::world::World;

use crate::graphics::geometry::Geometry;
use crate::graphics::material::BlendMode;
use crate::graphics::material::Material;
use crate::graphics::mesh::Mesh;
use crate::graphics::ribbon_shaders::FRAGMENT_SHADER;
use crate::graphics::ribbon_shaders::VERTEX_SHADER;
use crate::graphics::vectors::*;

pub struct Ribbon {
    mesh: Entity,
    geometry: Entity,
    pub spine: Vec<Point2d>,
    pub width: f64,
    pub closed: bool,
}

impl Ribbon {
    pub fn new(world: &mut World) -> Self {
        let geometry = world.spawn(Geometry::new()).id();

        let mut material_component = Material::new(VERTEX_SHADER, FRAGMENT_SHADER);
        material_component.set_blending(BlendMode::SourceOver);
        let material = world.spawn(material_component).id();

        let mut mesh_component = Mesh::new(geometry, material);
        mesh_component.visible = false;
        mesh_component.set_vec4("color", Vector4f::new(0.0, 0.4, 0.6, 1.0));
        let mesh = world.spawn(mesh_component).id();

        Self {
            mesh,
            geometry,
            spine: Vec::new(),
            width: 5000.0,
            closed: true,
        }
    }

    pub fn hide(&self, world: &mut World) {
        let mesh = world.get_mut::<Mesh>(self.mesh).unwrap().into_inner();
        mesh.visible = false;
    }

    pub fn show(&self, world: &mut World) {
        let mesh = world.get_mut::<Mesh>(self.mesh).unwrap().into_inner();
        mesh.visible = true;
    }

    pub fn set_render_order(&self, world: &mut World, render_order: i32) {
        let mesh = world.get_mut::<Mesh>(self.mesh).unwrap().into_inner();
        mesh.render_order = render_order;
    }

    pub fn update(&mut self, world: &mut World, gl: &glow::Context) {
        let points = &self.spine;

        if points.len() < 2 {
            self.hide(world);
            return;
        }

        self.show(world);

        let mut positions = Vec::new();
        let mut indices = Vec::new();

        // Helper function to add a 3D point to positions
        let add_point = |positions: &mut Vec<f32>, p: Point2d| {
            positions.extend_from_slice(&[p.x as f32, p.y as f32, 0.0]);
        };

        // Helper function to add a triangle to indices
        let add_triangle = |indices: &mut Vec<u32>, a: u32, b: u32, c: u32| {
            indices.extend_from_slice(&[a, b, c]);
        };

        let count = if self.closed {
            points.len() - 1
        } else {
            points.len()
        };

        let upper = if self.closed { count + 1 } else { count };

        for i in 0..upper {
            let prev = points[(i + count - 1) % count];
            let curr = points[i % count];
            let next = points[(i + 1) % count];

            let mut dir1 = (curr - prev).normalize();
            let mut dir2 = (next - curr).normalize();

            if !self.closed && i == 0 {
                dir1 = dir2;
            }

            if !self.closed && i == count - 1 {
                dir2 = dir1;
            }

            let normal = Vector2d::new(-dir1.y, dir1.x);

            let miter_dir = (dir1 + dir2).normalize();
            let miter_dir = Vector2d::new(-miter_dir.y, miter_dir.x);

            let miter_length = 0.5 * self.width / normal.dot(&miter_dir);

            let base = positions.len() as u32 / 3;
            add_point(&mut positions, curr + miter_dir * miter_length);
            add_point(&mut positions, curr - miter_dir * miter_length);
            if i > 0 {
                add_triangle(&mut indices, base - 2, base, base - 1);
                add_triangle(&mut indices, base - 1, base, base + 1);
            }
        }

        // Create new geometry with the calculated data
        let mut new_geometry = Geometry::new();
        new_geometry.positions = positions;
        new_geometry.indices = indices;
        new_geometry.replace(world, gl, self.geometry);
    }
}
