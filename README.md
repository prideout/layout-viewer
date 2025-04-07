# Layout Viewer

Rust library and CLI tool for rendering GDSII layouts with OpenGL / WebGL or
converting them to SVG format.

Includes a [web app](https://prideout.net/layout-viewer/) that allows users
to drop a GDSII file for local viewing. All work is performed in the browser
and no data is sent to the server. In fact there is no server, it's just
a static site hosted with GitHub Pages.

## Modules

- `core` contains the data model and core logic.
  - Defines a set of ECS components.  
- `graphics` is a simple WebGL rendering library.
  - Provides abstractions similar to libraries like THREE and Filament.
  - Knows nothing about circuits or app behavior.
  - All objects except **Renderer** can be constructed without a WebGL
    context.
- `webui` defines a set of Yew components.
  - The UI for the web application lives here.
  - Components with the **Page** suffix are navigation targets.
- `cli` provides a command-line interface and simple native window target.
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
trunk serve --open
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

Loader chunks should be based on a combo of struct & shape counts
  - 4004 should update the status

`get_or_create_layer` should maybe use a SystemState for storing queries
  - In fact there lots of queries sprinkled throughout the codebase

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

## An Entity-Component-Query architecture in Rust.

`layout-viewer` is my little web app for viewing for integrated circuit
layouts, written in Rust.

The app's data model required quite a bit of cross-referencing due to the heavy
use of instancing in photolithography. I knew that strong ownership semantics
would be difficult to manage, so I decided to use an ID-based architecture,
where various objects simply refer to each other without any ownership. Naturally
my thoughts turned to an ECS, which in turn made me think of Bevy. Bevy has a
nice ECS that lives in an isolated crate without any of the game engine stuff
that I didn't need.

Most of you know what an ECS is. En bref: **entities** are IDs representing
objects, **components** are pure data bundles attached to entities (with no
behavior), and **systems** process entities with specific component combinations
to implement game logic. ECS architectures really shine for certain types of
games and simulations.

The "systems" in Bevy's ECS also have sophisticated features for scheduling and
parallelism. It also has features like commands, events, and more. This felt
like more than what I needed. My app is not a simulation or a game and there's
only one thread since it targets WASM. Plus I wanted to have tight control over
scheduling and async behavior myself. So I ended up using only entities,
components, and queries. Sometimes I learn best from the ground up, so this thin
slice of Bevy seemed like a good place to start. Maybe in the future I'll
explore using systems and more standard best practices for Bevy.

### Hierarchical instancing

Here's a brief overview of some vocabulary used in the world of integrated
circuit design. You'll find these terms in specification like GDSII and OASIS.

- Cell (aka Structure): Defines a building block with polygons and child cells
- Instance (aka Reference): Placement of a cell with specific transformation
- Polygon: Closed shape representing a physical element in a certain layer
- Layer: Fabrication plane with a certain thickness and material (e.g., silicon, metal, or an insulator)

Incidentally the hierarchical instancing implied by this data model is similar to what's common for BIM and the AEC industry, like what we deal with at Arcol. For example, two instanced doorknobs might be used to create a door component, and a set of instanced doors might be used in a building floor, and an instanced set of floors could form a skyscraper.

Hierarchical instancing has first-class support in complex scene description
formats like Pixar's USD format, but not in simpler scene graphs like what you'd
fine in glTF and ThreeJS. glTF and ThreeJS _do_ support instancing; e.g. in
Three, the same `Geometry` can be shared amongst several `Mesh` objects.
However, local transforms are not shared between disparate parts of the scene
graph. For example, if a user moves a doorknob within a door component, then
they should see all door instances in the world automatically get updated. This
kind of behavior is not a core feature in the ThreeJS scene graph.

That's okay for my project though because I'm using ThreeJS, I've got my own
simple WebGL renderer. Here's what an abridged version of my data model looks
like:

```rust
// TODO
```

### Bevy's query objects

Query objects are really neat. Behind the scenes, Bevy maintains multiple lists
of entities: one for each active combination of components. When
a component is added or removed from an entity, the entity gets moved from
one list to another. These lists incidentally are called "archetypes".

Write about single queries, tag components, etc.
