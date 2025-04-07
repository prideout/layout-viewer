# Layout Viewer

Rust library and CLI tool for rendering GDSII layouts with OpenGL / WebGL or
converting them to SVG format.

Includes a [web app](https://prideout.net/layout-viewer/) that allows users
to drop a GDSII file for local viewing. All work is performed in the browser
and no data is sent to the server. In fact there is no server, it's just
a static site hosted with GitHub Pages.

## Modules

- `core` is the data model for the application.
  - All types related to **Project** are defined here.
  - May include caches and acceleration structures.
- `graphics` is a simple WebGL rendering library.
  - Provides abstractions similar to libraries like THREE and Filament.
  - Knows nothing about circuits or app behavior.
  - All objects except **Renderer** can be constructed without a WebGL
    context.
- `webui` defines a set of Yew components.
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

- bevy_ecs: Entity-component-system
- gds21: GDSII parser
- svg: SVG generation
- wasm-bindgen: WebAssembly bindings
- anyhow: Error handling

## Next tasks

- Resurrect CLI app
- Move app_controller and hover_effect into `core`
- Move cli, generate_svg, and app_window into `cli`
- Create `graphics/vectors.rs` and shrink top of camera/components/controller
- Remove old_core

Loader chunks should be based on a combo of struct & shape counts
  - 4004 should update the status

`get_or_create_layer` should maybe use a SystemState for storing queries
  - In fact there lots of queries sprinkled throughout the codebase

Write blog post: The ECQ architecture: Entity-Component-Query.

  For fun I wrote a web-based viewer for integrated circuit layouts in Rust. Its
  data model required quite a bit of cross-referencing (heavy use of
  instancing). I knew that strong ownership semantics would be difficult to
  manage, so I decided to use an id-based architecture.
  Naturally my thoughts turned to an ECS, which in turn made me think of Bevy.
  Bevy has a nice ECS that lives in an isolated crate without any of the game
  engine stuff that I don't need.
  
  However even Bevy's ECS alone felt a bit more than what I needed; my app is
  not a simulation or a game, and there's only one thread since it targets WASM.
  No need for sophisticated scheduling or "systems". I ended up using only
  entities, components, and queries.
  
  I'm pretty happy with how it turned out so I thought I'd write about it.

  Query objects are really neat. Behind the scenes, Bevy maintains multiple
  lists of entities: one for each query object. And it updates each one when you
  add or remove components from the world. The premise is that adding and
  removing entities is nuch less frequent than iterating over them.

Look for memory leaks.

The user should be able to choose a cell definition as "current root"

ShapeDefinition boxes and CellReference boxes should appear according to pointer position,
but only for direct children of the current root.

The user should be able to drag cells, but only cells that happen to be direct children of the current root.

The user should be able to drag shapes, but only shapes that happen to be direct children of the current root.

Double clicking a cell or shape should set that cell's definition as root.

-----

Sep jobs (see the static.yml at root)

Zooming out should constrain pan (internally optional)

Test / fix the app on mobile devices. (pointer events, not mouse events; hide sidebar)

Try to fix the status text issues. Use Evan's state diagram generator.

Implement array refs.

Integer grid.

Performance / smooth zoom.

Camera constraints / frame upon selection / "Reset view".

Van Wijk interpolation / marquee selection.

Better CI (Build & Run CLI, Doc Tests, Unit Tests).

## Code formatting

```bash
cargo +nightly fmt
```

or a single file:

```bash
rustfmt +nightly --edition 2024 src/core/loader.rs
```