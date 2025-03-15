use anyhow::{anyhow, Result};
use gds21::GdsLibrary;
use wasm_bindgen::prelude::*;

#[derive(Debug)]
pub struct LayoutStats {
    pub cell_count: usize,
    pub total_polygons: usize,
    pub total_paths: usize,
    pub total_srefs: usize,
    pub total_arefs: usize,
    pub total_texts: usize,
    pub total_nodes: usize,
    pub total_boxes: usize,
}

#[wasm_bindgen]
pub struct Layout {
    library: GdsLibrary,
    stats: LayoutStats,
}

#[wasm_bindgen]
impl Layout {
    pub fn from_bytes(data: &[u8]) -> Result<Layout, JsValue> {
        // NOTE: this library should take a slice.
        let library = GdsLibrary::from_bytes(data.to_vec())
            .map_err(|e| JsValue::from_str(&format!("Failed to parse GDSII: {}", e)))?;

        let mut stats = LayoutStats {
            cell_count: library.structs.len(),
            total_polygons: 0,
            total_paths: 0,
            total_srefs: 0,
            total_arefs: 0,
            total_texts: 0,
            total_nodes: 0,
            total_boxes: 0,
        };

        // Count elements in each cell
        for gds_struct in &library.structs {
            for element in &gds_struct.elems {
                match element {
                    gds21::GdsElement::GdsBoundary(_) => stats.total_polygons += 1,
                    gds21::GdsElement::GdsPath(_) => stats.total_paths += 1,
                    gds21::GdsElement::GdsStructRef(_) => stats.total_srefs += 1,
                    gds21::GdsElement::GdsArrayRef(_) => stats.total_arefs += 1,
                    gds21::GdsElement::GdsTextElem(_) => stats.total_texts += 1,
                    gds21::GdsElement::GdsNode(_) => stats.total_nodes += 1,
                    gds21::GdsElement::GdsBox(_) => stats.total_boxes += 1,
                }
            }
        }

        Ok(Layout { library, stats })
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

// Non-wasm public functions
impl Layout {
    pub fn stats(&self) -> &LayoutStats {
        &self.stats
    }
    pub fn name(&self) -> &str {
        &self.library.name
    }
    pub fn process_gds_file(data: &[u8]) -> Result<Self> {
        Self::from_bytes(data).map_err(|e| {
            anyhow!(
                "Failed to process GDSII file: {}",
                e.as_string().unwrap_or_default()
            )
        })
    }
}
