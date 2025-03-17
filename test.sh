trunk build

cargo run --release --quiet --bin layout-viewer ./assets/gds/caravel.gds
cargo run --release --quiet --bin layout-viewer ./assets/gds/test/SimplePath.gds

cargo run --quiet --bin layout-viewer ./assets/gds/intel-4004.gds           ./assets/gds/intel-4004.svg
cargo run --quiet --bin layout-viewer ./assets/gds/mos-6502.gds             ./assets/gds/mos-6502.svg
cargo run --quiet --bin layout-viewer ./assets/gds/test/SimpleRotation.gds  ./assets/gds/test/SimpleRotation.svg
cargo run --quiet --bin layout-viewer ./assets/gds/test/SimpleMirror.gds    ./assets/gds/test/SimpleMirror.svg
cargo run --quiet --bin layout-viewer ./assets/gds/test/SimpleBoth.gds      ./assets/gds/test/SimpleBoth.svg
cargo run --quiet --bin layout-viewer ./assets/gds/test/SimpleRotation2.gds ./assets/gds/test/SimpleRotation2.svg
cargo run --quiet --bin layout-viewer ./assets/gds/test/SimpleBoth2.gds     ./assets/gds/test/SimpleBoth2.svg
cargo run --quiet --bin layout-viewer ./assets/gds/trilomix-sky130.gds      ./assets/gds/trilomix-sky130.svg
cargo run --quiet --bin layout-viewer ./assets/gds/trilomix-example.gds     ./assets/gds/trilomix-example.svg # has arefs

open -a 'Google Chrome' ./assets/gds/intel-4004.svg
open -a 'Google Chrome' ./assets/gds/mos-6502.svg
open -a 'Google Chrome' ./assets/gds/test/SimpleRotation.svg
open -a 'Google Chrome' ./assets/gds/test/SimpleMirror.svg
open -a 'Google Chrome' ./assets/gds/test/SimpleBoth.svg
open -a 'Google Chrome' ./assets/gds/test/SimpleRotation2.svg
open -a 'Google Chrome' ./assets/gds/test/SimpleBoth2.svg
open -a 'Google Chrome' ./assets/gds/trilomix-sky130.svg
open -a 'Google Chrome' ./assets/gds/trilomix-example.svg
