wasm-pack build --target web

# cargo run --release --quiet --bin layout-viewer ../public/caravel.gds # has paths

cargo run --quiet --bin layout-viewer ../public/intel-4004.gds
cargo run --quiet --bin layout-viewer ../public/mos-6502.gds
cargo run --quiet --bin layout-viewer ../public/test/collect_rename_au4.gds # multi-root
cargo run --quiet --bin layout-viewer ../public/test/collect_rename_au.gds
# cargo run --quiet --bin layout-viewer ../public/trilomix-example.gds # no paths but has arefs
# cargo run --quiet --bin layout-viewer ../public/trilomix-sky130.gds  # has some paths but no arefs
# cargo run --quiet --bin layout-viewer ../public/test/bug_121c.gds # has paths
# cargo run --quiet --bin layout-viewer ../public/test/arefs_skew1.gds

