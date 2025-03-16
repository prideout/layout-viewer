## Tasks

- Fix collect_rename_au.svg
- Get paths working.
- Get arefs working.
- Stub GlView, GlScene, GlMesh, GlGeometry, GlMaterial, all of which have ids.

Create WebGL triangles by calling `earcut_triangles_raw` on the geo polygons
and appending them to a VBO held in the layer.

For WebGL, create an app with trunk + Yew + glow (?) with a sidebar for layers.
Each layer will have a color swatch, visibility toggle, and opacity slider.

Use [https://docs.rs/crate/bvh](https://docs.rs/crate/bvh) for accelerated
picking. It uses nalgebra internally.

## Rendering procedure

https://docs.rs/geo/latest/geo/algorithm/affine_ops/struct.AffineTransform.html

1. Allocate a `Vec<RenderLayer>` where each RenderLayer holds a map from
   `CellRefId` to a `geo::Polygon`.
2. Starting at the root, traverse down the CellRef tree and update the affine
   transforms stored in each CellRef, where the root has the identity matrix.
   Populate the RenderLayer with the polygons of the cell. Also expand a global
   AABB.
3. To render SVG, go through the layers and emit a flat list of paths.
   Each layer should be an SVG `<g>` with 50% opacity.

## Sanity checks

```
wasm-pack build --target web
cargo run --bin layout-viewer ../public/trilomix-example.gds
cargo run --bin layout-viewer ../public/trilomix-sky130.gds
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

