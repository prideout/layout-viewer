use crate::Route;
use web_sys::DragEvent;
use yew::prelude::*;
use yew_router::prelude::*;

#[function_component(Home)]
pub fn home() -> Html {
    let _navigator = use_navigator().unwrap();

    let on_drop = Callback::from(move |event: DragEvent| {
        event.prevent_default();
        // if let Some(data_transfer) = event.data_transfer() {
        //     if let Some(files) = data_transfer.files() {
        //         if let Some(file) = files.get(0) {
        //             // Handle the dropped GDS file here
        //             log::info!("File dropped: {}", file.name());
        //         }
        //     }
        // }
    });

    let on_dragover = Callback::from(|event: DragEvent| {
        event.prevent_default();
    });

    html! {
        <div class="tile-container">
            <a href="https://github.com/prideout/layout-viewer"
               class="tile github-tile"
               target="_blank">
                <i class="fab fa-github"></i>
                <span class="tile-text">{"GitHub Repo"}</span>
            </a>

            <Link<Route> to={Route::Layout { id: "intel-4004".to_string() }} classes="tile">
                <i class="fas fa-microchip"></i>
                <span class="tile-text">{"Intel 4004"}</span>
            </Link<Route>>

            <Link<Route> to={Route::Layout { id: "mos-6502".to_string() }} classes="tile">
                <i class="fas fa-microchip"></i>
                <span class="tile-text">{"MOS Technology 6502"}</span>
            </Link<Route>>

            <Link<Route> to={Route::Layout { id: "caravel-harness".to_string() }} classes="tile">
                <i class="fas fa-microchip"></i>
                <span class="tile-text">{"Caravel Harness"}</span>
            </Link<Route>>

            <Link<Route> to={Route::Layout { id: "skywater-130".to_string() }} classes="tile">
                <i class="fas fa-industry"></i>
                <span class="tile-text">{"SkyWater 130"}</span>
            </Link<Route>>

            <div class="tile drop-tile"
                ondrop={on_drop}
                ondragover={on_dragover}>
                <i class="fas fa-upload"></i>
                <span class="tile-text">{"Drop GDS file"}</span>
            </div>
        </div>
    }
}
