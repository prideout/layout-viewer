#![allow(dead_code)]

use gds21::{GdsBoundary, GdsPath, GdsPoint, GdsStrans};
use geo::AffineTransform;

/// Simple integer ID for cells, guaranteed to be unique within a project.
///
/// 0 is reserved for root cells, which don't actually have a `Cell` object
/// because they always live at the origin with an identity transform.
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct CellId(pub(crate) usize);

/// Simple integer ID for cell defs, guaranteed to be unique within a project.
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct CellDefId(pub(crate) usize);

/// Instance of a [CellDef].
/// Corresponds to a SRef or a sub-instance of an ARef.
///
/// Yes there's a naming conflict with std::cell::Cell, but this isn't quite
/// the same thing as a GDSII SRef so I don't want to call it a StructRef.
pub(crate) struct Cell {
    pub cell_id: CellId,
    pub cell_def_id: CellDefId,
    pub xy: GdsPoint,
    pub local_transform: Option<GdsStrans>,
    pub world_transform: AffineTransform, // derived from local_transform by traversing the hierarchy
    pub visible: bool,
}

/// Instanceable definition of a cell.
/// Corresponds to a single GDSII struct.
#[derive(Debug, Clone)]
pub(crate) struct CellDef {
    pub cell_def_id: CellDefId,
    pub instances_of_self: Vec<CellId>,
    pub boundary_elements: Vec<GdsBoundary>,
    pub path_elements: Vec<GdsPath>,
    pub cell_elements: Vec<CellId>,
}

impl CellDef {
    pub fn new(cell_def_id: CellDefId) -> Self {
        Self {
            cell_def_id,
            instances_of_self: Vec::new(),
            boundary_elements: Vec::new(),
            path_elements: Vec::new(),
            cell_elements: Vec::new(),
        }
    }
} 