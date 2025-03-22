# Layout Viewer

Rust library and CLI tool for rendering GDSII layouts with OpenGL / WebGL or
converting them to SVG format.

The library can be used both natively and as a WebAssembly module.

## Modules

- `core` is the data model for the application.
  - All types related to **Project** are defined here.
  - May include caches and acceleration structures.
- `graphics` is a simple WebGL rendering library.
  - Provides abstractions similar to libraries like THREE and Filament.
  - Knows nothing about circuits or app behavior.
  - All objects except **Renderer** can be constructed without a WebGL
    context.
- `components` defines a set of Yew components.
  - The UI for the web application lives here.
  - Components with the **Page** suffix are navigation targets.
- `rsutils` are utilities that you could imagine being a part of **std**.
  - Nothing here should know about circuits or the app.

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

- Simplify `update_outline_mesh`, no stroke, just draw the polygon with half opacity.
  - Remove earcutr and i_overlay

Resizing window should not reset the camera.

Use fp64 in the graphics layer.

Test / fix the app on mobile devices. (pointer events, not mouse events; hide sidebar)

Implement array refs.

Performance / smooth zoom.

Camera constraints / frame upon selection / "Reset view".

Van Wijk interpolation / marquee selection.

Better CI (Build & Run CLI, Doc Tests, Unit Tests).
