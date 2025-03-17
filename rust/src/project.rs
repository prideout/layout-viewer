#![allow(dead_code)]

use anyhow::{anyhow, Result};
use gds21::{GdsLibrary, GdsPoint, GdsStrans};
use geo::{AffineTransform, Coord};
use indexmap::IndexMap;
use wasm_bindgen::prelude::*;
use nalgebra::Vector4;

use crate::{
    bounds::BoundingBox,
    cells::{Cell, CellDef, CellDefId, CellId},
    layer::Layer,
    string_interner::StringInterner,
    svg_backend,
    id_map::IdMap,
};

#[derive(Debug)]
pub struct LayoutStats {
    pub struct_count: usize,
    pub polygon_count: usize,
    pub path_count: usize,
    pub sref_count: usize,
    pub aref_count: usize,
    pub text_count: usize,
    pub node_count: usize,
    pub box_count: usize,
}

#[wasm_bindgen]
pub struct Project {
    cells: IdMap<CellId, Cell>,
    cell_defs: IndexMap<CellDefId, CellDef>,
    render_layers: Vec<Layer>,
    highest_layer: i16,
    stats: LayoutStats,
    interner: StringInterner,
    bounds: BoundingBox,
}

impl Project {
    pub fn from_bytes(data: &[u8]) -> Result<Project> {
        let data = data.to_vec();

        let library =
            GdsLibrary::from_bytes(data).map_err(|e| anyhow!("Failed to parse GDSII: {}", e))?;

        let mut stats = LayoutStats {
            struct_count: library.structs.len(),
            polygon_count: 0,
            path_count: 0,
            sref_count: 0,
            aref_count: 0,
            text_count: 0,
            node_count: 0,
            box_count: 0,
        };

        // Build stats
        let mut highest_layer = 0;
        for gds_struct in &library.structs {
            for element in &gds_struct.elems {
                match element {
                    gds21::GdsElement::GdsBoundary(boundary) => {
                        highest_layer = highest_layer.max(boundary.layer);
                        stats.polygon_count += 1;
                    }
                    gds21::GdsElement::GdsPath(path) => {
                        highest_layer = highest_layer.max(path.layer);
                        stats.path_count += 1;
                    }
                    gds21::GdsElement::GdsStructRef(_) => stats.sref_count += 1,
                    gds21::GdsElement::GdsArrayRef(_) => stats.aref_count += 1,
                    gds21::GdsElement::GdsTextElem(_) => stats.text_count += 1,
                    gds21::GdsElement::GdsNode(_) => stats.node_count += 1,
                    gds21::GdsElement::GdsBox(_) => stats.box_count += 1,
                }
            }
        }

        let mut interner = StringInterner::new();
        let mut cells = IdMap::new();
        let mut cell_defs: IndexMap<CellDefId, CellDef> = IndexMap::new();

        let add_cell = |cells: &mut IdMap<CellId, Cell>,
                       cell_defs: &mut IndexMap<CellDefId, CellDef>,
                       interner: &mut StringInterner,
                       name: &str,
                       xy: &GdsPoint,
                       strans: &Option<GdsStrans>| {
            let cell_def_id = CellDefId(interner.intern(name));
            let cell = Cell {
                cell_id: CellId(0), // Will be set by IdMap
                cell_def_id,
                xy: xy.clone(),
                local_transform: strans.clone(),
                visible: true,
                world_transform: AffineTransform::identity(),
            };
            let cell_id = cells.create_id(cell);
            cell_defs
                .entry(cell_def_id)
                .or_insert(CellDef::new(cell_def_id))
                .instances_of_self
                .push(cell_id);
            cell_id
        };

        for cell in &library.structs {
            let cell_def_id = CellDefId(interner.intern(&cell.name));
            let mut cell_def = cell_defs
                .get(&cell_def_id)
                .unwrap_or(&CellDef::new(cell_def_id))
                .clone();
            for elem in &cell.elems {
                match elem {
                    gds21::GdsElement::GdsStructRef(sref) => {
                        cell_def.cell_elements.push(add_cell(&mut cells, &mut cell_defs, &mut interner, &sref.name, &sref.xy, &sref.strans));
                    }
                    gds21::GdsElement::GdsArrayRef(aref) => {
                        // TODO: I think this isn't a great way to handle it.
                        // Just make one cell ref; the entire array will correspond to a single geo Polygon.
                        let count = aref.cols * aref.rows;
                        for _ in 0..count {
                            let id = add_cell(&mut cells, &mut cell_defs, &mut interner, &aref.name, &aref.xy[0], &aref.strans);
                            // TODO: array refs are not yet implemented, hide them for now
                            cells.get_mut(&id).unwrap().visible = false;
                        }
                    }
                    gds21::GdsElement::GdsBoundary(boundary) => {
                        cell_def.boundary_elements.push(boundary.clone());
                    }
                    gds21::GdsElement::GdsPath(path) => {
                        cell_def.path_elements.push(path.clone());
                    }
                    gds21::GdsElement::GdsTextElem(_) => {
                        // TODO: at least emit a log message
                    }
                    gds21::GdsElement::GdsNode(_) => {
                        // TODO: at least emit a log message
                    }
                    gds21::GdsElement::GdsBox(_) => {
                        // TODO: at least emit a log message
                    }
                }
            }
            cell_defs.insert(cell_def_id, cell_def);
        }

        let mut project = Project {
            stats,
            interner,
            cells,
            cell_defs,
            render_layers: Vec::new(),
            highest_layer,
            bounds: BoundingBox::new(),
        };

        project.update_world_transforms();
        project.update_render_layers();

        Ok(project)
    }

    pub fn stats(&self) -> &LayoutStats {
        &self.stats
    }

    pub fn highest_layer(&self) -> i16 {
        self.highest_layer
    }

    pub fn struct_name(&self, cell_def_id: CellDefId) -> &str {
        self.interner.get(cell_def_id.0)
    }

    pub fn find_roots(&self) -> Vec<CellDefId> {
        self
            .cell_defs
            .iter()
            .filter(|(_, cell_def)| cell_def.instances_of_self.is_empty())
            .map(|(cell_def_id, _)| *cell_def_id)
            .collect()
    }

    pub fn update_world_transforms(&mut self) {
        let roots = self.find_roots();
        let identity = &AffineTransform::identity();
        for cell_def_id in roots {
            let cell_ids = self.cell_defs[&cell_def_id].cell_elements.clone();
            for cell_id in cell_ids {
                self.update_world_transforms_recurse(cell_id, identity);
            }
        }
    }

    pub fn update_render_layers(&mut self) {
        self.render_layers.clear();
        
        // Create layers with unique colors
        for i in 0..=self.highest_layer {
            let hue = (i as f32) / ((self.highest_layer + 1) as f32);
            let (r, g, b) = hsv_to_rgb(hue, 0.8, 0.8);
            let mut layer = Layer::new();
            layer.color = Vector4::new(r, g, b, 0.5);
            self.render_layers.push(layer);
        }

        let root_id = CellId(0);
        let identity = &AffineTransform::identity();
        for cell_def_id in self.find_roots() {
            let cell_def = self.cell_defs.get(&cell_def_id).unwrap();
            for boundary in &cell_def.boundary_elements {
                let layer = &mut self.render_layers[boundary.layer as usize];
                layer.add_boundary_element(root_id, boundary, identity);
            }
            for path in &cell_def.path_elements {
                let layer = &mut self.render_layers[path.layer as usize];
                layer.add_path_element(root_id, path, identity);
            }
            let cell_ids = self.cell_defs[&cell_def_id].cell_elements.clone();
            for cell_id in cell_ids {
                self.update_render_layers_recurse(cell_id);
            }
        }

        // Update bounds for each layer and the overall project
        self.bounds = BoundingBox::new();
        for layer in &mut self.render_layers {
            layer.update_bounds();
            if !layer.bounds.is_empty() {
                self.bounds.encompass(&layer.bounds);
            }
        }
    }

    fn update_render_layers_recurse(&mut self, cell_id: CellId) {
        let cell = self.cells.get(&cell_id).unwrap();
        if !cell.visible {
            return;
        }
        let transform = &cell.world_transform;
        let cell_def = self.cell_defs.get(&cell.cell_def_id).unwrap();
        for boundary in &cell_def.boundary_elements {
            let layer = &mut self.render_layers[boundary.layer as usize];
            layer.add_boundary_element(cell_id, boundary, transform);
        }
        for path in &cell_def.path_elements {
            let layer = &mut self.render_layers[path.layer as usize];
            layer.add_path_element(cell_id, path, transform);
        }
        let cell_ids = self.cell_defs[&cell.cell_def_id].cell_elements.clone();
        for cell_id in cell_ids {
            self.update_render_layers_recurse(cell_id);
        }
    }

    fn update_world_transforms_recurse(
        &mut self,
        cell_id: CellId,
        parent_transform: &AffineTransform,
    ) {
        let cell = self.cells.get_mut(&cell_id).unwrap();

        let translate = AffineTransform::translate(cell.xy.x as f64, cell.xy.y as f64);
        let mut rotate = AffineTransform::identity();
        let mut scale = AffineTransform::identity();

        if let Some(local_transform) = &cell.local_transform {
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

        let transform = scale
            .compose(&rotate)
            .compose(&translate)
            .compose(parent_transform);

        cell.world_transform = transform;

        let cell_ids = self.cell_defs[&cell.cell_def_id].cell_elements.clone();
        for cell_id in cell_ids {
            self.update_world_transforms_recurse(cell_id, &transform);
        }
    }

    pub fn render_layers(&self) -> &[Layer] {
        &self.render_layers
    }

    pub fn bounds(&self) -> &BoundingBox {
        &self.bounds
    }
}

// Helper function to convert HSV to RGB
fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (f32, f32, f32) {
    let h = h * 6.0;
    let i = h.floor();
    let f = h - i;
    let p = v * (1.0 - s);
    let q = v * (1.0 - s * f);
    let t = v * (1.0 - s * (1.0 - f));

    match i as i32 % 6 {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    }
}

#[wasm_bindgen]
impl Project {
    pub fn from_bytes_js(data: &[u8]) -> Result<Project, JsValue> {
        Project::from_bytes(data)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse GDSII: {}", e)))
    }

    pub fn to_svg_js(&self) -> Result<String, JsValue> {
        Ok(svg_backend::generate_svg(&self.render_layers))
    }
}
