# Layout Viewer

Rust library and CLI tool for rendering GDSII layouts with OpenGL / WebGL or
converting them to SVG format.

The library can be used both natively and as a WebAssembly module.

## Usage examples

```bash
# Open a GL window with the Intel 4004 chip:
cargo run --quiet --bin layout-viewer -- --gl assets/gds/intel-4004.gds

# Open a much larger GDS file using a release build:
cargo run --release --quiet --bin layout-viewer -- --gl assets/gds/caravel.gds

# Generate a SVG file:
cargo run --quiet --bin layout-viewer -- --gl assets/gds/mos-6502.gds mos-6502.svg

# Deploy a local web server:
trunk serve
```

## Limitations

- Arefs are ignored (but we might fix this)
- Magnification of elements is ignored.
- The "absolute" flag is ignored for magnitudes and angles.
- Text, Nodes, and Boxes are ignored.

## Dependencies

- gds21: GDSII parser
- svg: SVG generation
- wasm-bindgen: WebAssembly bindings
- anyhow: Error handling

## Next tasks

Choose one of these hit testing strategies:

- [RTree](https://docs.rs/rstar/latest/rstar/trait.RTreeObject.html) with geo polygons
- RTree with triangles
- [BVH](https://docs.rs/crate/bvh) with geo polygons
- BVH with triangles

Leaning towards RTree with polygons:

- No need to build wrappers like a list of "Triangle" objects
- Can support band boxes (rectangular selection)

---

For the selection effect, create an outline triangle buffer on the fly using the
stroke feature in `i_overlay`.

Test / fix the app on mobile devices.

The 6502 has bugs? I see sprinkled squares outside the bounds of the chip.

Implement array refs.

Performance / smooth zoom

Camera constraints / frame upon selection / "Reset view".

Van Wijk interpolation / marquee selection.

Better CI (Build & Run CLI, Doc Tests, Unit Tests, Clippy, Formatting)
