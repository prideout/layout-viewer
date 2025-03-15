wasm-pack build --target web

echo "\n4004:"
cargo run --quiet --bin layout-viewer ../public/intel-4004.gds

echo "\n6502:"
cargo run --quiet --bin layout-viewer ../public/mos-6502.gds

echo "\ntrilomix-example:"
cargo run --quiet --bin layout-viewer ../public/trilomix-example.gds

echo "\ntrilomix-sky130:"
cargo run --quiet --bin layout-viewer ../public/trilomix-sky130.gds

echo "\ncaravel:"
cargo run --release --quiet --bin layout-viewer ../public/caravel.gds
