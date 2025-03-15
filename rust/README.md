# Layout Viewer

A Rust library and CLI tool for processing GDSII layout files and converting
them to SVG format. The library can be used both natively and as a WebAssembly
module.

## Not implemented

- Arefs are ignored (but we might fix this)
- Magnification is ignored.
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

## License

MIT
