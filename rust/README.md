# Layout Viewer

Rust library and CLI tool for rendering GDSII layouts with OpenGL / WebGL
or converting them to SVG format.

The library can be used both natively and as a WebAssembly module.

## Tasks

```bash
cargo run --quiet --bin layout-viewer -- --gl ../public/intel-4004.gds
cargo run --release --quiet --bin layout-viewer -- --gl ../public/caravel.gds
```

Add scroll zooming using zoom-to-cursor.

Create a web app with trunk and Yew. It should have two routes: a home route and
a layouts/{id} route.

On the home page there should a vertical column of large tile buttons, centered
in the window. From top to bottom the tiles should say: "GitHub Repo" (with
github icon), "Intel 4004", "MOS Technology 6502", "Caravel Harness", "SkyWater
130", and "Drop GDS file".

The "Drop GDS File" tile is a drag-and-drop area that should change color when
the user hovers a file over it.  It should change to green if you hover a GDS or
a red if you hover anything else.  If you drop a GDS file, it should navigate to
the layouts/{id} route with the file loaded.  It can be clicked only if the user
has dropped a file.

On the layout page, there should be a sidebar on the right that shows a list of
layers. Each layer has a color swatch, visibility toggle, and opacity slider.
The central area of the app should be a filled a GL canvas that we can render to
using glow and our `GlRenderer` class.  Above the layer list are a few buttons:
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

