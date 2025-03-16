wasm-pack build --target web

# cargo run --release --quiet --bin layout-viewer ../public/caravel.gds # has paths

cargo run --quiet --bin layout-viewer ../public/intel-4004.gds ../public/intel-4004.svg
cargo run --quiet --bin layout-viewer ../public/mos-6502.gds ../public/mos-6502.svg
cargo run --quiet --bin layout-viewer ../public/test/SimpleRotation.gds ../public/test/SimpleRotation.svg
cargo run --quiet --bin layout-viewer ../public/test/SimpleMirror.gds ../public/test/SimpleMirror.svg
cargo run --quiet --bin layout-viewer ../public/test/SimpleBoth.gds ../public/test/SimpleBoth.svg
cargo run --quiet --bin layout-viewer ../public/test/SimpleRotation2.gds ../public/test/SimpleRotation2.svg
cargo run --quiet --bin layout-viewer ../public/test/SimpleBoth2.gds ../public/test/SimpleBoth2.svg

# cargo run --quiet --bin layout-viewer ../public/test/collect_rename_au.gds ../public/test/collect_rename_au.svg
# cargo run --quiet --bin layout-viewer ../public/trilomix-sky130.gds ../public/trilomix-sky130.svg  # has some paths but no arefs
# cargo run --quiet --bin layout-viewer ../public/test/bug_121c.gds # has paths
# cargo run --quiet --bin layout-viewer ../public/trilomix-example.gds # no paths but has arefs
# cargo run --quiet --bin layout-viewer ../public/test/arefs_skew1.gds

