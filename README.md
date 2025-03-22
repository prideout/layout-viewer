# Layout Viewer

Rust library and CLI tool for rendering GDSII layouts with OpenGL / WebGL or
converting them to SVG format.

The library can be used both natively and as a WebAssembly module.

## Modules

- `core` is the data model for the application.
  - No application logic, UI, or graphics code is allowed here.
  - Everything related to a "project" is defined here.
  - May include caches and acceleration structures.
- `graphics` is a simple WebGL rendering library.
  - Provides abstractions similar to libraries like THREE and Filament.
  - Knows nothing about circuits or app behavior.
  - Any graphics object except **Renderer** can be constructed without a WebGL
    context.
- `components` is a list of Yew components for the web app.
  - The UI for the web app lives here.
  - Components with the `Page` suffix are navigation targets.
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

Go back to making layer indices be actual indices, and just filter out empty
layers in the UI and/or when assigning colors.

Get rid of `screen_to_world`, use `unproject` instead.

Orientation fix. (use camera's `up` vector)

Hover highlighting. Create an outline triangle buffer on the fly using the
stroke feature in `i_overlay`.

Use fp64 in the graphics layer.

Test / fix the app on mobile devices. (pointer events, not mouse events; hide sidebar)

Implement array refs.

Performance / smooth zoom.

Camera constraints / frame upon selection / "Reset view".

Van Wijk interpolation / marquee selection.

Better CI (Build & Run CLI, Doc Tests, Unit Tests).
