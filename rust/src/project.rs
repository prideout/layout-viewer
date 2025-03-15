use anyhow::{anyhow, Result};
use gds21::GdsLibrary;
use std::{collections::HashMap, iter::repeat_with};
use wasm_bindgen::prelude::*;

use crate::interner::{CellId, StringInterner};

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

#[derive(Eq, Ord, PartialEq, PartialOrd)]
pub struct CellRefId(String);
impl CellRefId {
    fn new(id: String) -> Self {
        Self(id)
    }
}

#[wasm_bindgen]
pub struct Project {
    library: GdsLibrary,
    stats: LayoutStats,
    interner: StringInterner,
    cells: HashMap<CellId, Vec<CellId>>,
}

impl Project {
    pub fn generate_random_id(&self) -> CellRefId {
        let id: String = repeat_with(fastrand::alphanumeric).take(10).collect();
        CellRefId::new(id)
    }

    pub fn from_bytes(data: &[u8]) -> Result<Project> {
        // NOTE: gds21 is not being idiomatic here (should take a slice).
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
        for gds_struct in &library.structs {
            for element in &gds_struct.elems {
                match element {
                    gds21::GdsElement::GdsBoundary(_) => stats.polygon_count += 1,
                    gds21::GdsElement::GdsPath(_) => stats.path_count += 1,
                    gds21::GdsElement::GdsStructRef(_) => stats.sref_count += 1,
                    gds21::GdsElement::GdsArrayRef(_) => stats.aref_count += 1,
                    gds21::GdsElement::GdsTextElem(_) => stats.text_count += 1,
                    gds21::GdsElement::GdsNode(_) => stats.node_count += 1,
                    gds21::GdsElement::GdsBox(_) => stats.box_count += 1,
                }
            }
        }

        let mut interner = StringInterner::new();
        let mut cells: HashMap<CellId, Vec<CellId>> = HashMap::new();

        for cell in &library.structs {
            let cell_idx = interner.intern(&cell.name);
            for elem in &cell.elems {
                if let gds21::GdsElement::GdsStructRef(sref) = elem {
                    let cell_id = interner.intern(&sref.name);
                    cells.entry(cell_id).or_default().push(cell_idx);
                }
                if let gds21::GdsElement::GdsArrayRef(aref) = elem {
                    let call_id = interner.intern(&aref.name);
                    cells.entry(call_id).or_default().push(cell_idx);
                }
            }
        }

        let project = Project {
            library,
            stats,
            interner,
            cells,
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

    pub fn is_root_cell(&self, cell_name: &str) -> bool {
        if let Some(id) = self.interner.get_id(cell_name) {
            !self.cells.contains_key(&id)
        } else {
            true // If cell doesn't exist, consider it a root
        }
    }

    pub fn reference_count(&self, cell_name: &str) -> usize {
        if let Some(id) = self.interner.get_id(cell_name) {
            self.cells.get(&id).map_or(0, |v| v.len())
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
