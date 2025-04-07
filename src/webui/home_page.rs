use crate::webui::app::Route;
use crate::webui::toast::ToastContainer;
use crate::webui::toast::ToastManager;

use wasm_bindgen_futures::JsFuture;
use web_sys::DragEvent;
use yew::prelude::*;
use yew_router::prelude::*;

pub struct HomePage {
    dropped_file: Option<(String, Vec<u8>)>,
    is_dragging: bool,
    toast_manager: ToastManager,
    is_dark_theme: bool,
}

pub enum HomeMsg {
    FileDropped(String, Vec<u8>),
    NavigateToViewer,
    DragOver(bool),
    RemoveToast(usize),
}

impl Component for HomePage {
    type Message = HomeMsg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            dropped_file: None,
            is_dragging: false,
            toast_manager: ToastManager::new(),
            is_dark_theme: false,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            HomeMsg::FileDropped(name, content) => {
                self.dropped_file = Some((name, content));
                self.is_dragging = false;
                true
            }
            HomeMsg::NavigateToViewer => {
                if let Some((name, content)) = self.dropped_file.take() {
                    // Store the file content in a global state or context
                    // For now, we'll use a simple static variable
                    unsafe {
                        DROPPED_FILE = Some((name, content));
                    }
                    // Navigate to the viewer route
                    let navigator = ctx.link().navigator().unwrap();
                    navigator.push(&Route::Viewer {
                        id: "dropped-file".to_string(),
                    });
                } else {
                    self.toast_manager
                        .show("Drag and drop a valid GDS file.".to_string());
                }
                true
            }
            HomeMsg::DragOver(is_dragging) => {
                self.is_dragging = is_dragging;
                true
            }
            HomeMsg::RemoveToast(id) => {
                self.toast_manager.remove(id);
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let ondrop = {
            let link = ctx.link().clone();
            Callback::from(move |e: DragEvent| {
                e.prevent_default();
                if let Some(data_transfer) = e.data_transfer() {
                    if let Some(files) = data_transfer.files() {
                        if let Some(file) = files.get(0) {
                            let name = file.name();
                            let array_buffer = file.array_buffer();
                            let link = link.clone();
                            wasm_bindgen_futures::spawn_local(async move {
                                if let Ok(buffer) = JsFuture::from(array_buffer).await {
                                    let array = js_sys::Uint8Array::new(&buffer);
                                    let content = array.to_vec();
                                    link.send_message(HomeMsg::FileDropped(name, content));
                                }
                            });
                        }
                    }
                }
            })
        };

        let ondragover = {
            let link = ctx.link().clone();
            Callback::from(move |e: DragEvent| {
                e.prevent_default();
                if let Some(data_transfer) = e.data_transfer() {
                    // Tell the browser we accept the drop
                    data_transfer.set_drop_effect("copy");
                    link.send_message(HomeMsg::DragOver(true));
                }
            })
        };

        let ondragleave = {
            let link = ctx.link().clone();
            Callback::from(move |e: DragEvent| {
                e.prevent_default();
                link.send_message(HomeMsg::DragOver(false));
            })
        };

        let onclick = ctx.link().callback(|_| HomeMsg::NavigateToViewer);

        let drop_text = if let Some((name, _)) = &self.dropped_file {
            name.clone()
        } else {
            "Drop GDS".to_string()
        };

        let on_remove_toast = ctx.link().callback(HomeMsg::RemoveToast);

        html! {
            <div class={classes!("home-page-container", if self.is_dark_theme { "dark-theme" } else { "light-theme" })}>
                <div class="tile-container">
                    <a href="https://github.com/prideout/layout-viewer"
                       class="tile"
                       target="_blank">
                        <i class="fab fa-github"></i>
                        <span class="tile-text">{"prideout/layout-viewer"}</span>
                    </a>
                    <Link<Route> to={Route::Viewer { id: "intel-4004".to_string() }} classes="tile">
                        <i class="fas fa-microchip"></i>
                        <span class="tile-text">{"Intel 4004"}</span>
                    </Link<Route>>
                    <Link<Route> to={Route::Viewer { id: "mos-6502".to_string() }} classes="tile">
                        <i class="fas fa-microchip"></i>
                        <span class="tile-text">{"MOS 6502"}</span>
                    </Link<Route>>
                    // <Link<Route> to={Route::Viewer { id: "caravel".to_string() }} classes="tile">
                    //     <i class="fas fa-microchip"></i>
                    //     <span class="tile-text">{"Caravel Harness"}</span>
                    // </Link<Route>>
                    <Link<Route> to={Route::Viewer { id: "trilomix-sky130".to_string() }} classes="tile">
                        <i class="fas fa-microchip"></i>
                        <span class="tile-text">{"SkyWater SKY130"}</span>
                    </Link<Route>>
                    <div
                        class={classes!(
                            "tile",
                            "drop-tile",
                            if self.dropped_file.is_some() { "drop-valid" } else { "" },
                            if self.is_dragging { "drop-valid" } else { "" }
                        )}
                        {ondrop}
                        {ondragover}
                        {ondragleave}
                        {onclick}
                    >
                        <i class="fas fa-file-upload"></i>
                        <span class="tile-text">{drop_text}</span>
                        <span class="tile-text" style="font-size: 0.8em;">{"Stays in browser"}</span>
                        <span class="tile-text" style="font-size: 0.8em;">{"Not sent to server"}</span>
                    </div>
                </div>
                <ToastContainer toasts={self.toast_manager.toasts().to_vec()} on_remove={on_remove_toast} />
            </div>
        }
    }
}

// Static storage for the dropped file
static mut DROPPED_FILE: Option<(String, Vec<u8>)> = None;

// Function to get and clear the dropped file
#[allow(static_mut_refs)]
pub fn take_dropped_file() -> Option<(String, Vec<u8>)> {
    unsafe { DROPPED_FILE.take() }
}

#[allow(static_mut_refs)]
pub fn has_dropped_file() -> bool {
    unsafe { DROPPED_FILE.is_some() }
}
