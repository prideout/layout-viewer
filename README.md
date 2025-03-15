## Tasks

- Dump stats about the SRef refl/mag/rotation, then sanity check with KLayout
- Find the max layer number (stored in Boundary and Path)
- Create a `TreeItem` struct that holds all SRef state and derived state for
  rendering purposes (probably just a `nalgebra::Matrix3`).
- Create a root `TreeItem` for the top cell. This is the only `TreeItem` that
  does not have corresponding SRef.
- Write `build_layers`
- Write `render_svg`

## Rendering procedure

1. Allocate a `Vec<RenderLayer>` where each RenderLayer will hold a map from
   `CellRefId` to a `geo::Polygon`.
2. Starting at the root, traverse down and update the 3x3 transforms stored 
   in each TreeItem, where the root has the identity matrix. Populate the
   RenderLayer with the polygons of the cell. Also expand a global AABB.
3. To render SVG, go through the layers and emit a flat list of paths.
   Each layer should be an SVG `<g>` with 50% opacity.
4. To create WebGL triangles, call `earcut_triangles_raw` on the polygons.

Might want to use [https://docs.rs/crate/bvh](https://docs.rs/crate/bvh) for
accelerated picking.

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

