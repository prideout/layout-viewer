#![allow(dead_code)]

use anyhow::{anyhow, Result};
use gds21::{GdsBoundary, GdsLibrary, GdsPath, GdsPoint, GdsStrans};
use geo::AffineTransform;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

use crate::interner::StringInterner;

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

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct CellDefId(usize);

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct CellId(usize);

/// Instance of a [CellDef].
/// Corresponds to a SRef or a sub-instance of an ARef.
struct Cell {
    cell_id: CellId,
    cell_def_id: CellDefId,
    xy: GdsPoint,
    local_transform: Option<GdsStrans>,
    world_transform: AffineTransform,
    visible: bool,
}

/// Instanceable definition of a cell.
/// Corresponds to a single GDSII struct.
#[derive(Debug, Clone)]
struct CellDef {
    cell_def_id: CellDefId,
    instances_of_self: Vec<CellId>,
    boundary_elements: Vec<GdsBoundary>,
    path_elements: Vec<GdsPath>,
    // cell_elements: Vec<CellRefId>,
}

impl CellDef {
    fn new(cell_def_id: CellDefId) -> Self {
        Self {
            cell_def_id,
            instances_of_self: Vec::new(),
            boundary_elements: Vec::new(),
            path_elements: Vec::new(),
            // cell_elements: Vec::new(),
        }
    }
}

#[wasm_bindgen]
pub struct Project {
    library: GdsLibrary,
    stats: LayoutStats,
    interner: StringInterner,
    cells: HashMap<CellId, Cell>,
    cell_defs: HashMap<CellDefId, CellDef>,
    highest_layer: i16,
    next_cell_id: CellId,
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
        let mut cells: HashMap<CellId, Cell> = HashMap::new();
        let mut cell_defs: HashMap<CellDefId, CellDef> = HashMap::new();
        let mut next_cell_id = CellId(1);

        let add_cell = |cell_id: CellId,
                        cell_defs: &mut HashMap<CellDefId, CellDef>,
                        cells: &mut HashMap<CellId, Cell>,
                        interner: &mut StringInterner,
                        name: &str,
                        xy: &GdsPoint,
                        strans: &Option<GdsStrans>| {
            let cell_def_id = CellDefId(interner.intern(name));
            cells.insert(
                cell_id,
                Cell {
                    cell_id,
                    cell_def_id,
                    xy: xy.clone(),
                    local_transform: strans.clone(),
                    visible: true,
                    world_transform: AffineTransform::identity(),
                },
            );
            cell_defs
                .entry(cell_def_id)
                .or_insert(CellDef::new(cell_def_id))
                .instances_of_self
                .push(cell_id);
            CellId(cell_id.0 + 1)
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
                        next_cell_id = add_cell(
                            next_cell_id,
                            &mut cell_defs,
                            &mut cells,
                            &mut interner,
                            &sref.name,
                            &sref.xy,
                            &sref.strans,
                        );
                    }
                    gds21::GdsElement::GdsArrayRef(aref) => {
                        let count = aref.cols * aref.rows;
                        for _ in 0..count {
                            let id = next_cell_id;
                            next_cell_id = add_cell(
                                id,
                                &mut cell_defs,
                                &mut cells,
                                &mut interner,
                                &aref.name,
                                &aref.xy[0], // TODO: use the correct xy
                                &aref.strans,
                            );
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

        let project = Project {
            library,
            stats,
            interner,
            cells,
            cell_defs,
            highest_layer,
            next_cell_id,
        };

        Ok(project)
    }

    pub fn stats(&self) -> &LayoutStats {
        &self.stats
    }

    pub fn name(&self) -> &str {
        &self.library.name
    }

    pub fn library(&self) -> &GdsLibrary {
        &self.library
    }

    pub fn highest_layer(&self) -> i16 {
        self.highest_layer
    }

    /// Returns true if the given struct is not instantiated by any other struct.
    pub fn is_root_cell(&self, cell_name: &str) -> bool {
        self.reference_count(cell_name) == 0
    }

    /// Returns the number of times the given struct is referenced (instantiated).
    pub fn reference_count(&self, cell_name: &str) -> usize {
        if let Some(id) = self.interner.get_id(cell_name) {
            self.cell_defs
                .get(&CellDefId(id))
                .map_or(0, |v| v.instances_of_self.len())
        } else {
            0
        }
    }
}

#[wasm_bindgen]
impl Project {
    pub fn from_bytes_js(data: &[u8]) -> Result<Project, JsValue> {
        Project::from_bytes(data)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse GDSII: {}", e)))
    }

    pub fn to_svg(&self) -> Result<String, JsValue> {
        // This is a placeholder for SVG generation
        // TODO: Implement actual SVG generation using the svg crate
        let doc = svg::Document::new()
            .set("viewBox", (0, 0, 1000, 1000))
            .set("width", "1000")
            .set("height", "1000");

        Ok(doc.to_string())
    }
}
