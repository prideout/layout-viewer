# Layout Viewer

A Rust library and CLI tool for processing GDSII layout files and converting them to SVG format. The library can be used both natively and as a WebAssembly module.

## Features

- Parse GDSII binary files
- Generate SVG output
- Display layout statistics (cell count, polygons, paths, references)
- WebAssembly support for browser-based usage

## Dev usage

```bash
cargo run --bin layout-viewer -- --help
```

## Installation

```bash
cargo install --path .
```

## Usage

### Command Line Interface

```bash
layout-viewer input.gds output.svg
```

This will:
1. Read and parse the GDSII file
2. Display statistics about the layout
3. Generate an SVG file

### As a Library

```rust
use layout_viewer::Layout;

// Read GDSII data
let gds_data: Vec<u8> = /* your GDSII binary data */;
let layout = Layout::process_gds_file(&gds_data)?;

// Get statistics
println!("{}", layout.get_stats());

// Generate SVG
let svg_content = layout.to_svg()?;
```

### WebAssembly Usage

The library can be compiled to WebAssembly and used in web applications. Build the WebAssembly module with:

```bash
wasm-pack build --target web
```

## Dependencies

- gds21: GDSII parser
- svg: SVG generation
- wasm-bindgen: WebAssembly bindings
- anyhow: Error handling

## License

MIT 