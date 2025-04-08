use bevy_ecs::entity::Entity;
use bevy_ecs::query::QueryState;
use bevy_ecs::world::World;
use geo::Contains;
use rstar::RTree;
use rstar::RTreeObject;

use crate::core::components::Hovered;
use crate::core::components::Layer;
use crate::core::components::LayerMaterial;
use crate::core::components::LayerMesh;
use crate::core::components::ShapeInstance;
use crate::core::hover_effect::HoverEffect;
use crate::core::hover_effect::HoverParams;
use crate::core::layer_proxy::LayerProxy;
use crate::core::rtree::RTreeItem;
use crate::graphics::bounds::BoundingBox;
use crate::graphics::camera::Camera;
use crate::graphics::geometry::Geometry;
use crate::graphics::material::BlendMode;
use crate::graphics::material::Material;
use crate::graphics::mesh::Mesh;
use crate::graphics::renderer::Renderer;
use crate::graphics::vectors::*;
use crate::graphics::viewport::Viewport;

/// Encapsulates high-level application logic common to all platforms.
pub struct AppController {
    window_size: (u32, u32),
    renderer: Renderer,
    camera: Camera,
    world: World,
    mut_layer_query: QueryState<(Entity, &'static mut Layer)>,
    layer_query: QueryState<&'static Layer>,
    is_dragging: bool,
    last_mouse_pos: Option<(u32, u32)>,
    zoom_speed: f64,
    needs_render: bool,
    hover_effect: HoverEffect,
    rtree: RTree<RTreeItem>,
}

pub enum Theme {
    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    Light,
    Dark,
}

#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
impl AppController {
    pub fn new(renderer: Renderer, physical_width: u32, physical_height: u32) -> Self {
        let camera = Camera::new(Point3d::new(0.0, 0.0, 0.0), 128.0, 128.0, -1.0, 1.0);

        let mut world = World::new();

        let hover_effect = HoverEffect::new(&mut world);

        let layer_query = QueryState::new(&mut world);

        let mut_layer_query = QueryState::new(&mut world);

        Self {
            window_size: (physical_width, physical_height),
            renderer,
            camera,
            world,
            layer_query,
            mut_layer_query,
            is_dragging: false,
            last_mouse_pos: None,
            zoom_speed: 0.05,
            needs_render: true,
            hover_effect,
            rtree: RTree::new(),
        }
    }

    pub fn set_world(&mut self, mut world: World) {
        if world.id() == self.world.id() {
            return;
        }

        self.hover_effect = HoverEffect::new(&mut world);
        self.hover_effect.set_render_order(&mut world, 9999);
        self.renderer.on_new_world(&mut world);
        self.world = world;
        self.layer_query = QueryState::new(&mut self.world);
        self.mut_layer_query = QueryState::new(&mut self.world);

        let mut world_bounds = BoundingBox::new();
        for layer in self.layer_query.iter_mut(&mut self.world) {
            world_bounds.encompass(&layer.world_bounds);
        }

        log::info!("World bounds: {:?}", world_bounds);
        log::info!("Window size: {:?}", self.window_size);

        self.camera.fit_to_bounds(self.window_size, world_bounds);

        self.render();

        let mut rtree_items = Vec::new();
        let mut shape_query = self.world.query::<(Entity, &ShapeInstance)>();
        for (entity, shape_instance) in shape_query.iter(&self.world) {
            rtree_items.push(RTreeItem {
                shape_instance: entity,
                aabb: shape_instance.world_polygon.envelope(),
            });
        }
        self.rtree = RTree::bulk_load(rtree_items);
    }

    pub fn handle_mouse_press(&mut self, x: u32, y: u32) {
        self.is_dragging = true;
        self.last_mouse_pos = Some((x, y));
    }

    pub fn handle_mouse_release(&mut self) {
        self.is_dragging = false;
        self.last_mouse_pos = None;
    }

    pub fn handle_mouse_move(&mut self, x: u32, y: u32) {
        if self.is_dragging {
            if let Some((last_x, last_y)) = self.last_mouse_pos {
                let p1 = self.screen_to_world(x, y);
                let p0 = self.screen_to_world(last_x, last_y);
                let dx = p1.0 - p0.0;
                let dy = p1.1 - p0.1;

                let mut pos = self.camera.position;
                pos.x -= dx;
                pos.y -= dy;
                self.camera.position = pos;
                self.render();
            }
            self.last_mouse_pos = Some((x, y));
        }

        // Convert screen coordinates to world space
        let (world_x, world_y) = self.screen_to_world(x, y);

        // Find the single entity that has the Hovered component, if it exists.
        let hovered_entity = self
            .world
            .query::<(Entity, &Hovered)>()
            .get_single(&self.world)
            .ok()
            .map(|(entity, _)| entity)
            .unwrap_or(Entity::PLACEHOLDER);

        if let Some(hit) = self.pick_cell(world_x, world_y) {
            if hit.shape_instance != hovered_entity {
                if hovered_entity != Entity::PLACEHOLDER {
                    self.world.entity_mut(hovered_entity).remove::<Hovered>();
                }
                self.world.entity_mut(hit.shape_instance).insert(Hovered);
                self.hover_effect.show(HoverParams {
                    shape_instance: hit.shape_instance,
                    world: &mut self.world,
                    gl: self.renderer.gl(),
                });
                self.render();
            }
        } else if hovered_entity != Entity::PLACEHOLDER {
            self.hover_effect.hide(&mut self.world);
            self.world.entity_mut(hovered_entity).remove::<Hovered>();
            self.render();
        }
    }

    pub fn handle_mouse_wheel(&mut self, x: u32, y: u32, delta: f64) {
        // Ignore very small deltas that might be touchpad bounce
        const MIN_DELTA: f64 = 0.01;
        if delta.abs() < MIN_DELTA {
            return;
        }

        // Convert screen coordinates to world space before zoom
        let (world_x, world_y) = self.screen_to_world(x, y);

        // Calculate zoom factor (positive delta = zoom in, negative = zoom out)
        // Clamp delta to avoid extreme zoom changes
        let clamped_delta = delta.clamp(-1.0, 1.0);
        let zoom_factor = if clamped_delta > 0.0 {
            1.0 - self.zoom_speed
        } else {
            1.0 + self.zoom_speed
        };

        // Update camera size (zoom)
        self.camera.width *= zoom_factor;
        self.camera.height *= zoom_factor;

        // Convert the same screen coordinates to world space after zoom
        let (new_world_x, new_world_y) = self.screen_to_world(x, y);

        // Adjust camera position to keep cursor point stable
        self.camera.position.x += world_x - new_world_x;
        self.camera.position.y += world_y - new_world_y;

        self.render();
    }

    pub fn handle_mouse_leave(&mut self) {
        let hovered_entity = self
            .world
            .query::<(Entity, &Hovered)>()
            .get_single(&self.world)
            .ok()
            .map(|(entity, _)| entity)
            .unwrap_or(Entity::PLACEHOLDER);

        if hovered_entity != Entity::PLACEHOLDER {
            self.hover_effect.hide(&mut self.world);
            self.world.entity_mut(hovered_entity).remove::<Hovered>();
            self.render();
        }
    }

    /// Requests a render to occur during the next tick.
    pub fn render(&mut self) {
        self.needs_render = true;
    }

    /// Unconditionally called every 16 ms, returns "true" if the framebuffer
    /// was refreshed.
    pub fn tick(&mut self) -> bool {
        if !self.needs_render {
            return false;
        }

        let width = 5.0 * self.camera.width / self.window_size.0 as f64;
        self.hover_effect
            .update_stroke_width(width, &mut self.world, self.renderer.gl());

        self.renderer.render(&mut self.world, &self.camera);
        self.renderer.check_gl_error("Scene render");
        self.needs_render = false;
        true // Frame was rendered
    }

    pub fn resize(&mut self, physical_width: u32, physical_height: u32) {
        self.window_size = (physical_width, physical_height);
        self.renderer.set_viewport(Viewport {
            left: 0.0,
            top: 0.0,
            width: physical_width as f64,
            height: physical_height as f64,
        });
        let window_aspect = physical_width as f64 / physical_height as f64;
        self.camera.height = self.camera.width / window_aspect;

        self.renderer.render(&mut self.world, &self.camera);
        self.renderer.check_gl_error("Scene render");
    }

    pub fn destroy(&mut self) {
        let mut geo_query = self.world.query::<&mut Geometry>();
        for mut geo in geo_query.iter_mut(&mut self.world) {
            geo.destroy(self.renderer.gl());
        }

        let mut mat_query = self.world.query::<&mut Material>();
        for mut mat in mat_query.iter_mut(&mut self.world) {
            mat.destroy(self.renderer.gl());
        }

        // TODO: despawn the entities too
    }

    pub fn apply_theme(&mut self, theme: Theme) {
        let mut count = 0;
        for layer in self.layer_query.iter(&self.world) {
            if !layer.shape_instances.is_empty() {
                count += 1;
            }
        }
        let alpha = 1.0 / (count as f32);

        for (_, mut layer) in self.mut_layer_query.iter_mut(&mut self.world) {
            layer.color.w = alpha;
        }

        let mut mesh_query = self.world.query::<(&mut Mesh, &LayerMesh)>();
        for mut mesh in mesh_query.iter_mut(&mut self.world) {
            let color = match theme {
                Theme::Light => Vector4f::new(0.0, 0.0, 0.0, alpha),
                Theme::Dark => Vector4f::new(1.0, 1.0, 1.0, alpha),
            };
            mesh.0.set_vec4("color", color);
        }

        let mut mat_query = self.world.query::<(&mut Material, &LayerMaterial)>();
        let mut material = mat_query.single_mut(&mut self.world).0;
        match theme {
            Theme::Light => {
                material.set_blending(BlendMode::Additive);
            }
            Theme::Dark => {
                material.set_blending(BlendMode::SourceOver);
            }
        }

        self.render();
    }

    pub fn create_layer_proxies(&mut self) -> Vec<LayerProxy> {
        let mut layer_proxies = Vec::new();
        for (entity, layer) in self.mut_layer_query.iter(&self.world) {
            layer_proxies.push(LayerProxy::from_layer(entity, layer));
        }
        layer_proxies
    }

    pub fn update_layer(&mut self, layer_proxy: LayerProxy) {
        let mut layer = self
            .mut_layer_query
            .get_mut(&mut self.world, layer_proxy.entity)
            .unwrap()
            .1;
        layer_proxy.to_layer(&mut layer);
        let visible = layer.visible;
        let color = layer.color;
        let mesh = layer.mesh;

        let mut mesh = self.world.get_mut::<Mesh>(mesh).unwrap();
        mesh.set_vec4("color", color);
        mesh.visible = visible;
    }

    fn pick_cell(&self, x: f64, y: f64) -> Option<RTreeItem> {
        let point = geo::Point::new(x, y);
        let items = self.rtree.locate_all_at_point(&point);
        let mut result: Option<RTreeItem> = None;
        let mut result_layer_index = -i16::MAX;

        // Of all items whose AABB overlaps the query point, pick the one with
        // the highest layer index, but only if its layer is visible, and if its
        // polygon actually contains the point.

        for item in items {
            let shape_instance = self
                .world
                .get::<ShapeInstance>(item.shape_instance)
                .unwrap();

            if shape_instance.layer_index < result_layer_index {
                continue;
            }

            let layer = self.world.get::<Layer>(shape_instance.layer).unwrap();

            if !layer.visible {
                continue;
            }

            if !shape_instance.world_polygon.contains(&point) {
                continue;
            }

            result = Some(item.clone());
            result_layer_index = shape_instance.layer_index;
        }
        result
    }

    fn screen_to_world(&self, screen_x: u32, screen_y: u32) -> (f64, f64) {
        let ndc_x = (screen_x as f64 / self.window_size.0 as f64) * 2.0 - 1.0;
        let ndc_y = -((screen_y as f64 / self.window_size.1 as f64) * 2.0 - 1.0);
        let world = self.camera.unproject(Point3d::new(ndc_x, ndc_y, 0.0));
        (world.x, world.y)
    }
}

impl Drop for AppController {
    fn drop(&mut self) {
        self.destroy();
    }
}
