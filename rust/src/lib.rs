use anyhow::{anyhow, Result};
use gds21::GdsLibrary;
use wasm_bindgen::prelude::*;

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
            struct_count: library.structs.len(),
            polygon_count: 0,
            path_count: 0,
            sref_count: 0,
            aref_count: 0,
            text_count: 0,
            node_count: 0,
            box_count: 0,
        };

        // Count elements in each cell
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
    pub fn library(&self) -> &GdsLibrary {
        &self.library
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
