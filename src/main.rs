#[cfg(not(target_arch = "wasm32"))]
mod cli;

#[cfg(target_arch = "wasm32")]
fn main() {
    wasm_logger::init(wasm_logger::Config::new(log::Level::Info));
    yew::Renderer::<layout_viewer::App>::new().render();
}

#[cfg(not(target_arch = "wasm32"))]
fn main() -> anyhow::Result<()> {
    cli::run_cli()
}
