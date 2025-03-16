## Tasks

Get rid of Vec2d, it's only used by render_layer.

Create a file called `gl_backend.rs` that exports a fn called `generate_svg` that
takes a slice of layers and mutable `GlRenderer`. For each layer, it creates
a `GlMesh` and a `GlGeometry` to store triangles. For each polygon in the layer,
it calls `earcut_triangles_raw` on the polygon and appends the triangle verts
and indices to two growing arrays.  Coordinates are normalized such that bounding
box maps to [-1,+1] along the X axis.

Add a method to GlRenderer called render that takes a Scene ref and a CameraId.

Create a web app with trunk and Yew. There should be a sidebar on the right that
shows a list of layers. Each layer has a color swatch, visibility toggle,
and opacity slider. The central area of the app should be a filled a GL canvas
that we can render to using glow and our `GlRenderer` class.

Use [https://docs.rs/crate/bvh](https://docs.rs/crate/bvh) for accelerated
picking. It uses nalgebra internally.

Save array refs for last.

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

