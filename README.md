## Tasks

- Repair the wasm build
- Dump stats about the SRef refl/mag/rotation, then sanity check with KLayout
- Find the max layer number (stored in Boundary and Path)
- Create a `DagNode` struct that duplicates a StructRef + stores derived state
  for rendering purposes (such as 3x3 matrix). This is the DAG.
- Write `render_svg`

## Rendering plan

1. Create a DagNode for the top cell. This is the only DagNode that does not have
   corresponding SRef.
2. Allocate a `Vec<RenderLayer>` where each RenderLayer holds a list of
   CellRefId-GeoPolygon pairs.
2. Topological sort.
3. Starting at the top cell, traverse downstream and update a 3x3 matrix stored 
   in each DagNode, where the top cell has the identity matrix. Populate the
   RenderLayer with the polygons of the cell.

## Sanity checks

```
wasm-pack build --target web
cargo run --bin layout-viewer ../public/trilomix-example.gds
cargo run --bin layout-viewer ../public/trilomix-sky130.gds
cargo run --bin layout-viewer ../public/caravel.gds
cargo run --release --bin layout-viewer ../public/caravel.gds
```

## References

- https://www.artwork.com/gdsii/gdsii/page5.htm
- https://github.com/GraphiteEditor/Graphite/blob/master/node-graph/gcore/src/graphic_element/renderer.rs
- https://crates.io/crates/gds21

## Web app references

- https://jakearchibald.github.io/svgomg/
- https://github.com/bumbu/svg-pan-zoom
- https://docs.rs/specta/latest/specta/
- https://github.com/jakearchibald/svgomg/blob/1e1a1448f25761e7382cae5de2ba21f1e6ba439d/src/css/_global.scss#L16

