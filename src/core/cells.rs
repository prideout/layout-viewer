use crate::core::Element;
use crate::rsutils::IdMapKey;
use gds21::GdsPoint;
use gds21::GdsStrans;
use geo::AffineTransform;

/// Simple integer ID for cells, guaranteed to be unique within a project.
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct CellId(pub(crate) usize);

/// Simple integer ID for cell defs, guaranteed to be unique within a project.
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub struct CellDefId(pub(crate) usize);

impl IdMapKey for CellId {
    fn from_usize(id: usize) -> Self {
        CellId(id)
    }
}

/// Renderable instance of a [CellDef], positioned in the world.
pub struct Cell {
    pub cell_def_id: CellDefId,
    pub xy: GdsPoint,
    pub local_transform: Option<GdsStrans>,
    pub world_transform: AffineTransform, // derived from local_transform by traversing the hierarchy
    pub visible: bool,
    pub array: Option<ArrayProperties>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ArrayProperties {
    pub rows: i16,
    pub cols: i16,
    pub width: f64,
    pub height: f64,
}

/// Instanceable template definition of a cell.
/// Corresponds to a single GDSII struct.
pub struct CellDef {
    /// Polygon shapes and cached triangulations.
    pub elements: Vec<Element>,

    /// Instances that populate this cell.
    pub child_instances: Vec<CellId>,

    /// Present only for roots; holds a faux "ref" to self.
    /// This exists only to provide a starting point for recursion.
    pub root_instance: Option<CellId>,

    /// Instances of self, derived at load time.
    /// This is the reverse direction of [Cell::cell_def_id].
    /// NOTE: We are not using this, it could be probably be removed.
    pub instances: Vec<CellId>,
}

impl CellDef {
    pub fn new() -> Self {
        Self {
            instances: vec![],
            elements: Vec::new(),
            child_instances: Vec::new(),
            root_instance: None,
        }
    }
}

impl Default for CellDef {
    fn default() -> Self {
        Self::new()
    }
}
