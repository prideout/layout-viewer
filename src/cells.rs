use gds21::{GdsBoundary, GdsPath, GdsPoint, GdsStrans};
use geo::AffineTransform;

use crate::id_map::Id;

/// Simple integer ID for cells, guaranteed to be unique within a project.
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct CellId(pub(crate) usize);

/// Simple integer ID for cell defs, guaranteed to be unique within a project.
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct CellDefId(pub(crate) usize);

impl Id for CellId {
    fn from_usize(id: usize) -> Self {
        CellId(id)
    }
}

/// Renderable instance of a [CellDef], positioned in the world.
pub(crate) struct Cell {
    pub cell_def_id: CellDefId,
    pub xy: GdsPoint,
    pub local_transform: Option<GdsStrans>,
    pub world_transform: AffineTransform, // derived from local_transform by traversing the hierarchy
    pub visible: bool,
    pub array: Option<ArrayProperties>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub(crate) struct ArrayProperties {
    pub rows: i16,
    pub cols: i16,
    pub width: f64,
    pub height: f64,
}

/// Instanceable template definition of a cell.
/// Corresponds to a single GDSII struct.
#[derive(Debug, Clone)]
pub(crate) struct CellDef {
    pub boundary_elements: Vec<GdsBoundary>,
    pub path_elements: Vec<GdsPath>,
    pub cell_elements: Vec<CellId>,
    pub instances: Vec<CellId>,
    pub root_instance: Option<CellId>,
}

impl CellDef {
    pub fn new() -> Self {
        Self {
            instances: vec![],
            boundary_elements: Vec::new(),
            path_elements: Vec::new(),
            cell_elements: Vec::new(),
            root_instance: None,
        }
    }
}
