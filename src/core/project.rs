use crate::core::ArrayProperties;
use crate::core::Cell;
use crate::core::CellDef;
use crate::core::CellDefId;
use crate::core::CellId;
use crate::core::Element;
use crate::core::GdsSourceShape;
use crate::core::Layer;
use crate::graphics::BoundingBox;
use crate::rsutils::hsv_to_rgb;
use crate::rsutils::IdMap;
use crate::rsutils::StringInterner;
use anyhow::anyhow;
use anyhow::Result;
use gds21::GdsLibrary;
use gds21::GdsPoint;
use gds21::GdsStrans;
use geo::AffineTransform;
use geo::Contains;
use geo::Coord;
use geo::Point;
use nalgebra::Vector4;
use rstar::Envelope;
use rstar::PointDistance;
use rstar::RTree;
use rstar::RTreeObject;
use rstar::AABB;
use std::collections::BTreeMap;
use std::fmt::Debug;
use std::fmt::Formatter;
use std::fmt::{self};

/// Owns the data model for the application.
pub struct Project {
    cells: IdMap<CellId, Cell>,
    cell_defs: BTreeMap<CellDefId, CellDef>,
    layers: Vec<Layer>,
    highest_layer: i16,
    interner: StringInterner,
    bounds: BoundingBox,
    rtree: RTree<ElementRef>,
}

impl Project {
    pub fn from_bytes(data: &[u8]) -> Result<Project> {
        let data = data.to_vec();

        let library =
            GdsLibrary::from_bytes(data).map_err(|e| anyhow!("Failed to parse GDSII: {}", e))?;

        // Build stats
        let mut highest_layer = 0;
        for gds_struct in &library.structs {
            for element in &gds_struct.elems {
                match element {
                    gds21::GdsElement::GdsBoundary(boundary) => {
                        highest_layer = highest_layer.max(boundary.layer);
                    }
                    gds21::GdsElement::GdsPath(path) => {
                        highest_layer = highest_layer.max(path.layer);
                    }
                    _ => {}
                }
            }
        }

        let mut interner = StringInterner::new();
        let mut cells = IdMap::new();
        let mut cell_defs = BTreeMap::new();

        let add_cell = |cells: &mut IdMap<CellId, Cell>,
                        cell_defs: &mut BTreeMap<CellDefId, CellDef>,
                        interner: &mut StringInterner,
                        name: &str,
                        xy: &GdsPoint,
                        strans: &Option<GdsStrans>| {
            let cell_def_id = CellDefId(interner.intern(name));
            let cell = Cell {
                cell_def_id,
                xy: xy.clone(),
                local_transform: strans.clone(),
                visible: true,
                world_transform: AffineTransform::identity(),
                array: None,
            };
            let cell_id = cells.insert(cell);
            cell_defs
                .get_mut(&cell_def_id)
                .unwrap()
                .instances
                .push(cell_id);
            cell_id
        };

        for cell in &library.structs {
            let cell_def_id = CellDefId(interner.intern(&cell.name));
            cell_defs.insert(cell_def_id, CellDef::new());
        }

        for cell in &library.structs {
            let cell_def_id = CellDefId(interner.intern(&cell.name));
            let mut cell_def = cell_defs.remove(&cell_def_id).unwrap();
            for elem in &cell.elems {
                match elem {
                    gds21::GdsElement::GdsStructRef(sref) => {
                        cell_def.child_instances.push(add_cell(
                            &mut cells,
                            &mut cell_defs,
                            &mut interner,
                            &sref.name,
                            &sref.xy,
                            &sref.strans,
                        ));
                    }
                    gds21::GdsElement::GdsArrayRef(aref) => {
                        let id = add_cell(
                            &mut cells,
                            &mut cell_defs,
                            &mut interner,
                            &aref.name,
                            &aref.xy[0],
                            &aref.strans,
                        );

                        cell_def.child_instances.push(id);

                        let cols = aref.cols;
                        let rows = aref.rows;

                        // TODO: Is this correct?
                        let width = aref.xy[1].x as f64 - aref.xy[0].x as f64;
                        let height = aref.xy[2].y as f64 - aref.xy[0].y as f64;

                        // TODO: array refs are not yet implemented, hide them for now

                        let cell = cells.get_mut(&id).unwrap();
                        cell.visible = false;
                        cell.array = Some(ArrayProperties {
                            rows,
                            cols,
                            width,
                            height,
                        });
                    }
                    gds21::GdsElement::GdsBoundary(boundary) => {
                        let element = Element::from_boundary(boundary.clone());
                        cell_def.elements.push(element);
                    }
                    gds21::GdsElement::GdsPath(path) => {
                        let element = Element::from_path(path.clone());
                        cell_def.elements.push(element);
                    }
                    gds21::GdsElement::GdsTextElem(_) => {
                        // We do not support text elements yet, but they do
                        // occur so let's not spam the console with warnings.
                    }
                    gds21::GdsElement::GdsNode(_) => {
                        log::warn!("Node elements are not supported");
                    }
                    gds21::GdsElement::GdsBox(_) => {
                        log::warn!("Box elements are not supported");
                    }
                }
            }
            cell_defs.insert(cell_def_id, cell_def);
        }

        let mut project = Project {
            interner,
            cells,
            cell_defs,
            layers: Vec::new(),
            highest_layer,
            bounds: BoundingBox::new(),
            rtree: RTree::new(),
        };

        let roots = project.find_roots();
        for root in roots {
            let cell_def = project.cell_defs.get_mut(&root).unwrap();
            cell_def.root_instance = Some(project.cells.create_id());
        }

        project.update_world_transforms();
        project.update_layers();

        Ok(project)
    }

    pub fn highest_layer(&self) -> i16 {
        self.highest_layer
    }

    pub fn struct_name(&self, cell_def_id: CellDefId) -> &str {
        self.interner.get(cell_def_id.0)
    }

    pub fn find_roots(&self) -> Vec<CellDefId> {
        self.cell_defs
            .iter()
            .filter(|(_, cell_def)| cell_def.instances.is_empty())
            .map(|(cell_def_id, _)| *cell_def_id)
            .collect()
    }

    pub fn update_world_transforms(&mut self) {
        let roots = self.find_roots();
        let identity = &AffineTransform::identity();
        for cell_def_id in roots {
            let cell_ids = self.cell_defs[&cell_def_id].child_instances.clone();
            for cell_id in cell_ids {
                self.update_world_transforms_recurse(cell_id, identity);
            }
        }
    }

    pub fn update_layers(&mut self) {
        self.layers.clear();
        for i in 0..=self.highest_layer {
            self.layers.push(Layer::new(i));
        }

        let mut rtree_items = Vec::new();

        let identity = &AffineTransform::identity();
        for cell_def_id in self.find_roots() {
            let cell_def = self.cell_defs.get(&cell_def_id).unwrap();
            let root_id = cell_def.root_instance.unwrap();

            for element in &cell_def.elements {
                match element.shape {
                    GdsSourceShape::Boundary(ref boundary) => {
                        let layer = &mut self.layers[boundary.layer as usize];
                        layer.add_boundary_element(boundary, identity);
                        rtree_items.push(ElementRef {
                            aabb: layer.polygons.last().unwrap().envelope(),
                            layer: boundary.layer,
                            polygon: layer.polygons.len() - 1,
                            cell_id: root_id,
                        });
                    }
                    GdsSourceShape::Path(ref path) => {
                        let layer = &mut self.layers[path.layer as usize];
                        layer.add_path_element(path, identity);
                        rtree_items.push(ElementRef {
                            aabb: layer.polygons.last().unwrap().envelope(),
                            layer: path.layer,
                            polygon: layer.polygons.len() - 1,
                            cell_id: root_id,
                        });
                    }
                }
            }

            let cell_ids = self.cell_defs[&cell_def_id].child_instances.clone();
            for cell_id in cell_ids {
                self.update_layers_recurse(cell_id, &mut rtree_items);
            }
        }

        self.apply_light_scheme();

        // Update bounds for each layer and the overall project
        self.bounds = BoundingBox::new();
        let mut num_polygons = 0;
        for layer in &mut self.layers {
            layer.update_bounds();
            if !layer.bounds.is_empty() {
                self.bounds.encompass(&layer.bounds);
            }
            num_polygons += layer.polygons.len();
        }

        log::info!("--------------------------------");
        log::info!("Number of polygons: {}", num_polygons);

        self.rtree = RTree::bulk_load(rtree_items);
    }

    pub fn apply_rainbow_scheme(&mut self) {
        let mut count = 0;
        for layer in &self.layers {
            if !layer.polygons.is_empty() {
                count += 1;
            }
        }

        let mut i = 0;
        for layer in &mut self.layers {
            if layer.polygons.is_empty() {
                continue;
            }
            // Make the last layer white. To my eyes this looks somewhat better, aesthetically.
            if i == count - 1 {
                layer.color = Vector4::new(1.0, 1.0, 1.0, 0.5);
                break;
            }
            let hue = (i as f32) / (count as f32);
            let (r, g, b) = hsv_to_rgb(hue, 0.8, 0.8);
            layer.color = Vector4::new(r, g, b, 0.5);
            i += 1;
        }
    }

    pub fn apply_light_scheme(&mut self) {
        let mut count = 0;
        for layer in &self.layers {
            if !layer.polygons.is_empty() {
                count += 1;
            }
        }

        let alpha = 1.0 / (count as f32);

        for layer in &mut self.layers {
            if layer.polygons.is_empty() {
                continue;
            }
            layer.color = Vector4::new(0.0, 0.0, 0.0, alpha);
        }
    }

    fn update_layers_recurse(&mut self, cell_id: CellId, rtree_items: &mut Vec<ElementRef>) {
        let cell = self.cells.get(&cell_id).unwrap();
        if !cell.visible {
            return;
        }
        let transform = &cell.world_transform;
        let cell_def = self.cell_defs.get(&cell.cell_def_id).unwrap();
        for element in &cell_def.elements {
            match element.shape {
                GdsSourceShape::Boundary(ref boundary) => {
                    let layer = &mut self.layers[boundary.layer as usize];
                    layer.add_boundary_element(boundary, transform);
                    rtree_items.push(ElementRef {
                        aabb: layer.polygons.last().unwrap().envelope(),
                        layer: boundary.layer,
                        polygon: layer.polygons.len() - 1,
                        cell_id,
                    });
                }
                GdsSourceShape::Path(ref path) => {
                    let layer = &mut self.layers[path.layer as usize];
                    layer.add_path_element(path, transform);
                    rtree_items.push(ElementRef {
                        aabb: layer.polygons.last().unwrap().envelope(),
                        layer: path.layer,
                        polygon: layer.polygons.len() - 1,
                        cell_id,
                    });
                }
            }
        }

        let cell_ids = self.cell_defs[&cell.cell_def_id].child_instances.clone();
        for cell_id in cell_ids {
            self.update_layers_recurse(cell_id, rtree_items);
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

        let cell_ids = self.cell_defs[&cell.cell_def_id].child_instances.clone();
        for cell_id in cell_ids {
            self.update_world_transforms_recurse(cell_id, &transform);
        }
    }

    pub fn layers(&self) -> &[Layer] {
        &self.layers
    }

    pub fn layers_mut(&mut self) -> &mut [Layer] {
        &mut self.layers
    }

    pub fn bounds(&self) -> BoundingBox {
        self.bounds
    }

    pub fn pick_cell(&self, x: f64, y: f64) -> Option<ElementRef> {
        let point = Point::new(x, y);
        let items = self.rtree.locate_all_at_point(&point);
        let mut result: Option<ElementRef> = None;
        for item in items {
            if let Some(ref result) = result {
                if item.layer < result.layer {
                    continue;
                }
            }
            let layer = &self.layers[item.layer as usize];
            let polygon = &layer.polygons[item.polygon];
            if layer.visible && polygon.contains(&point) {
                result = Some(item.clone());
            }
        }
        result
    }
}

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

/// Identifies a unique instance of [Element].
#[derive(Clone)]
pub struct ElementRef {
    aabb: AABB<Point<f64>>,
    pub polygon: usize,
    pub layer: i16,
    pub cell_id: CellId,
}

impl Debug for ElementRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{ polygon {}, layer {}, cell_id {} }}",
            self.polygon, self.layer, self.cell_id.0
        )
    }
}

impl PartialEq for ElementRef {
    fn eq(&self, other: &Self) -> bool {
        self.polygon == other.polygon && self.layer == other.layer && self.cell_id == other.cell_id
    }
}

impl Eq for ElementRef {}

impl RTreeObject for ElementRef {
    type Envelope = AABB<Point<f64>>;

    fn envelope(&self) -> Self::Envelope {
        self.aabb
    }
}

impl PointDistance for ElementRef {
    fn distance_2(&self, point: &Point<f64>) -> f64 {
        self.aabb.distance_2(point)
    }

    fn contains_point(&self, point: &Point<f64>) -> bool {
        self.aabb.contains_point(point)
    }
}
