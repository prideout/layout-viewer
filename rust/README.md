# Layout Viewer

Rust library and CLI tool for rendering GDSII layouts with OpenGL / WebGL
or converting them to SVG format.

The library can be used both natively and as a WebAssembly module.

## Usage

```bash
cargo run --quiet --bin layout-viewer -- --gl ../public/intel-4004.gds
cargo run --release --quiet --bin layout-viewer -- --gl ../public/caravel.gds

trunk serve
```

## Tasks

On the layout/{id} page, there should be a GL canvas that fills the entire window that we can render to using glow and our `Renderer` class. There are two floating circular icon buttons in the top-left: a home button (which navigates the user back to the root route) and a github icon (which navigates to https://github.com/prideout/layout-viewer).
When the page initially loads and the canvas DOM element becomes available, we should create a gloo context (WebGL 2) and pass it into a Renderer constructor. We can store the Renderer in the page state.

On the layout page, there should be a sidebar on the right that shows a list of
layers. Each layer has a color swatch, visibility toggle, and opacity slider.
The central area of the app should be a filled a GL canvas that we can render to
using glow and our `GlRenderer` class. Above the layer list are a few buttons:
"Back home", "Enable picking", "Show all", and "Hide all".

Add BSD 3-clause and open source it.

Add blog post.

Default colors/opacities look terrible.

Use [https://docs.rs/crate/bvh](https://docs.rs/crate/bvh) for accelerated
picking. It uses nalgebra internally. Create an outline triangle buffer on the
fly using the stroke feature in i_overlay.

Save array refs for last.

## Limitations

- Arefs are ignored (but we might fix this)
- Magnification of elements is ignored.
- The "absolute" flag is ignored for magnitudes and angles.
- Text, Nodes, and Boxes are ignored.

## Dev usage

```bash
cargo run --bin layout-viewer -- --help
cargo run --bin layout-viewer ../public/trilomix-example.gds
```

## End users

```bash
cargo install --path .
layout-viewer input.gds output.svg
```

### WebAssembly usage

The library can be compiled to WebAssembly and used in web applications. Build
the WebAssembly module with:

```bash
wasm-pack build --target web
```

## Dependencies

- gds21: GDSII parser
- svg: SVG generation
- wasm-bindgen: WebAssembly bindings
- anyhow: Error handling

## References

- https://www.artwork.com/gdsii/gdsii/page5.htm
- https://github.com/GraphiteEditor/Graphite/blob/master/node-graph/gcore/src/graphic_element/renderer.rs
- https://crates.io/crates/gds21

## Web app references

- https://jakearchibald.github.io/svgomg/
- https://github.com/bumbu/svg-pan-zoom
- https://docs.rs/specta/latest/specta/
- https://github.com/jakearchibald/svgomg/blob/1e1a1448f25761e7382cae5de2ba21f1e6ba439d/src/css/_global.scss#L16
