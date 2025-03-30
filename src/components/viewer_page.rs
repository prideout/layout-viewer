use gloo::timers::callback::Timeout;
use serde::Serialize;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::window;
use web_sys::HtmlCanvasElement;
use web_sys::MouseEvent;
use web_sys::Request;
use web_sys::RequestInit;
use web_sys::Response;
use web_sys::WebGl2RenderingContext;
use web_sys::WheelEvent;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::app_controller::AppController;
use crate::app_controller::Theme;
use crate::components::take_dropped_file;
use crate::components::LayerProxy;
use crate::components::Route;
use crate::components::Sidebar;
use crate::components::ToastContainer;
use crate::components::ToastManager;
use crate::graphics::Renderer;
use crate::graphics::Scene;
use crate::performance_now;
use crate::rsutils::hex_to_rgb;
use crate::rsutils::rgb_to_hex;
use crate::rsutils::ResizeObserver;
use crate::Project;

#[derive(Properties, PartialEq)]
pub struct ViewerProps {
    pub id: String,
}

pub enum ViewerMsg {
    MousePress(u32, u32),
    MouseRelease,
    MouseMove(u32, u32),
    MouseWheel(u32, u32, f64),
    MouseLeave,
    GdsLoaded(Box<Project>),
    SetProject(Box<Project>),
    Render,
    Resize,
    Tick,
    RemoveToast(usize),
    UpdateLayer(LayerProxy),
    ToggleTheme,
}

pub struct ViewerPage {
    canvas_ref: NodeRef,
    controller: Option<AppController>,
    status: String,
    status_timestamp: f64,
    toast_manager: ToastManager,
    layer_proxies: Vec<LayerProxy>,
    is_dark_theme: bool,
}

impl ViewerPage {
    fn set_status(&mut self, status: &str) {
        log::info!("{} took {:.0}ms", self.status, performance_now!() - self.status_timestamp);
        self.status_timestamp = performance_now!();
        self.status = status.to_string();
    }

    fn clear_status(&mut self) {
        log::info!("{} took {:.0}ms", self.status, performance_now!() - self.status_timestamp);
        self.status.clear();
    }
}

impl Component for ViewerPage {
    type Message = ViewerMsg;
    type Properties = ViewerProps;

    fn create(ctx: &Context<Self>) -> Self {
        let canvas_ref = NodeRef::default();
        let controller = None;
        let toast_manager = ToastManager::new();
        let layer_proxies = Vec::new();
        
        // Read theme from local storage
        let is_dark_theme = if let Some(window) = window() {
            if let Some(storage) = window.local_storage().unwrap() {
                storage.get_item("dark_theme").unwrap_or(None)
                    .map(|s| s == "true")
                    .unwrap_or(false)
            } else {
                false
            }
        } else {
            false
        };

        // Check for dropped file
        if let Some((_name, content)) = take_dropped_file() {
            let link = ctx.link().clone();
            wasm_bindgen_futures::spawn_local(async move {
                match Project::from_bytes(&content) {
                    Ok(project) => {
                        link.send_message(ViewerMsg::GdsLoaded(Box::new(project)));
                    }
                    Err(_) => {
                        log::error!("Failed to parse dropped GDS.");
                    }
                }
            });
        } else if ctx.props().id == "dropped-file" {
            // No dropped file but on the dropped-file route, navigate back to home
            let navigator = ctx.link().navigator().unwrap();
            navigator.push(&Route::Home);
            log::info!("No dropped file found, redirecting to home page");
        }

        Self {
            canvas_ref,
            controller,
            status: "Fetching and reading GDS".to_string(),
            status_timestamp: performance_now!(),
            toast_manager,
            layer_proxies,
            is_dark_theme,
        }
    }

    fn destroy(&mut self, _ctx: &Context<Self>) {
        self.controller = None;
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let onmousedown = ctx.link().callback(|e: MouseEvent| {
            let x = e.offset_x() as u32;
            let y = e.offset_y() as u32;
            let scale = window().unwrap().device_pixel_ratio();
            let x = (x as f64) * scale;
            let y = (y as f64) * scale;
            ViewerMsg::MousePress(x as u32, y as u32)
        });

        let onmouseup = ctx.link().callback(|_| ViewerMsg::MouseRelease);

        let onmousemove = ctx.link().callback(|e: MouseEvent| {
            let x = e.offset_x() as u32;
            let y = e.offset_y() as u32;
            let scale = window().unwrap().device_pixel_ratio();
            let x = (x as f64) * scale;
            let y = (y as f64) * scale;
            ViewerMsg::MouseMove(x as u32, y as u32)
        });

        let onmouseleave = ctx.link().callback(|_| ViewerMsg::MouseLeave);

        let onwheel = ctx.link().callback(|e: WheelEvent| {
            e.prevent_default();
            let x = e.offset_x() as u32;
            let y = e.offset_y() as u32;
            let scale = window().unwrap().device_pixel_ratio();
            let x = (x as f64) * scale;
            let y = (y as f64) * scale;
            ViewerMsg::MouseWheel(x as u32, y as u32, e.delta_y())
        });

        let on_remove_toast = ctx.link().callback(ViewerMsg::RemoveToast);
        let update_layer = ctx.link().callback(ViewerMsg::UpdateLayer);
        let toggle_theme = ctx.link().callback(|_| ViewerMsg::ToggleTheme);

        html! {
            <>
                <div class={classes!("viewer-container", if self.is_dark_theme { "dark-theme" } else { "light-theme" })}>
                    <canvas
                        class="viewer-canvas"
                        ref={self.canvas_ref.clone()}
                        onmousedown={onmousedown}
                        onmouseup={onmouseup}
                        onmousemove={onmousemove}
                        onmouseleave={onmouseleave}
                        onwheel={onwheel}
                        style={"background-color: none;"}
                    />
                    <div class="floating-buttons">
                        <Link<Route> to={Route::Home} classes="floating-button">
                            <i class="fas fa-arrow-left fa-lg"></i>
                        </Link<Route>>
                        <button class="floating-button" onclick={toggle_theme}>
                            <i class={format!("fas fa-{} fa-lg", if self.is_dark_theme { "sun" } else { "moon" })}></i>
                        </button>
                        <span class="status-text">{self.status.clone()}</span>
                    </div>
                </div>
                <div class={classes!(if self.is_dark_theme { "dark-theme" } else { "light-theme" })}>
                    <Sidebar layers={self.layer_proxies.clone()} update_layer={update_layer} />
                </div>
                <ToastContainer toasts={self.toast_manager.toasts().to_vec()} on_remove={on_remove_toast} />
            </>
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if !first_render {
            return;
        }
        // Start GDS file fetch
        let id = ctx.props().id.clone();

        if id != "dropped-file" {
            let link = ctx.link().clone();
            wasm_bindgen_futures::spawn_local(async move {
                match fetch_gds_file(&id).await {
                    Ok(bytes) => {
                        let Ok(project) = Project::from_bytes(&bytes) else {
                            log::error!("Failed to parse fetched GDS.");
                            return;
                        };
                        link.send_message(ViewerMsg::GdsLoaded(Box::new(project)));
                    }
                    Err(e) => {
                        log::error!("Failed to fetch GDS file: {:?}", e);
                    }
                }
            });
        }

        // Get canvas and create WebGL context
        let Some(canvas) = self.canvas_ref.cast::<HtmlCanvasElement>() else {
            log::error!("Canvas not found");
            return;
        };

        #[derive(Serialize)]
        struct Options {
            alpha: bool,
            antialias: bool,
        }

        let options = serde_wasm_bindgen::to_value(&Options {
            alpha: true,
            antialias: true,
        }).unwrap();

        let gl: WebGl2RenderingContext = canvas
            .get_context_with_context_options("webgl2", &options)
            .unwrap()
            .unwrap()
            .dyn_into()
            .unwrap();

        // Create renderer with glow context
        let gl = glow::Context::from_webgl2_context(gl);
        let renderer = Renderer::new(gl);
        let scene = Scene::new();
        let width = canvas.client_width() as u32;
        let height = canvas.client_height() as u32;

        // Create controller
        let controller = AppController::new(renderer, scene, width, height);
        self.controller = Some(controller);

        // Set up resize observer
        let canvas_clone = canvas.clone();
        let link = ctx.link().clone();
        let resize_observer = ResizeObserver::new(move |_entries, _observer| {
            link.send_message(ViewerMsg::Resize);
        });
        resize_observer.observe(&canvas_clone);

        ctx.link().send_message(ViewerMsg::Tick);
        ctx.link().send_message(ViewerMsg::Render);
    }

    fn update(&mut self, context: &Context<Self>, msg: Self::Message) -> bool {
        let link = context.link().clone();
        let Some(controller) = &mut self.controller else {
            return false;
        };
        match msg {
            ViewerMsg::Resize => {
                if let Some(canvas) = self.canvas_ref.cast::<HtmlCanvasElement>() {
                    let width = canvas.client_width() as u32;
                    let height = canvas.client_height() as u32;
                    let scale = window().unwrap().device_pixel_ratio();
                    let width = width * scale as u32;
                    let height = height * scale as u32;
                    canvas.set_width(width);
                    canvas.set_height(height);
                    controller.resize(width, height);
                }
                false
            }
            ViewerMsg::Render => {
                controller.render();
                false
            }
            ViewerMsg::Tick => {
                controller.tick();
                let closure = Closure::wrap(Box::new(move || {
                    link.send_message(ViewerMsg::Tick);
                }) as Box<dyn FnMut()>);
                if let Some(window) = window() {
                    let _ = window.request_animation_frame(closure.as_ref().unchecked_ref());
                }
                closure.forget();
                false
            }
            ViewerMsg::MousePress(x, y) => {
                controller.handle_mouse_press(x, y);
                false
            }
            ViewerMsg::MouseRelease => {
                controller.handle_mouse_release();
                false
            }
            ViewerMsg::MouseMove(x, y) => {
                controller.handle_mouse_move(x, y);
                false
            }
            ViewerMsg::MouseWheel(x, y, delta) => {
                controller.handle_mouse_wheel(x, y, -delta);
                false
            }
            ViewerMsg::MouseLeave => {
                controller.handle_mouse_leave();
                false
            }
            ViewerMsg::GdsLoaded(mut project) => {
                if self.is_dark_theme {
                    project.apply_rainbow_scheme();
                }
                self.set_status("Triangulating polygons");
                let timeout = Timeout::new(1, move || link.send_message(ViewerMsg::SetProject(project)));
                timeout.forget();
                true
            }
            ViewerMsg::SetProject(project) => {
                let Some(controller) = &mut self.controller else {
                    log::error!("Controller not ready");
                    return false;
                };
                controller.set_project(*project);
                controller.apply_theme(if self.is_dark_theme { Theme::Dark } else { Theme::Light });
                self.toast_manager.show("Zoom and pan like a map".to_string());

                // Update layer proxies
                if let Some(project) = controller.project() {
                    self.layer_proxies = project
                        .layers()
                        .iter()
                        .enumerate()
                        .map(|(index, layer)| {
                            LayerProxy {
                                index,
                                visible: layer.visible,
                                opacity: layer.color.w,
                                color: rgb_to_hex(layer.color.x, layer.color.y, layer.color.z),
                                is_empty: layer.polygons.is_empty(),
                            }
                        })
                        .collect();
                }

                controller.render();
                self.clear_status();
                true
            }
            ViewerMsg::RemoveToast(id) => {
                self.toast_manager.remove(id);
                true
            }
            ViewerMsg::UpdateLayer(layer_proxy) => {
                let Some(controller) = &mut self.controller else {
                    return false;
                };
                let color = {
                    let Some(project) = controller.project_mut() else {
                        return false;
                    };
                    let Some(layer) = project.layers_mut().get_mut(layer_proxy.index) else {
                        return false;
                    };
                    layer.visible = layer_proxy.visible;
                    if let Some((r, g, b)) = hex_to_rgb(&layer_proxy.color) {
                        layer.color.w = layer_proxy.opacity;
                        layer.color.x = r;
                        layer.color.y = g;
                        layer.color.z = b;
                    }
                    layer.color
                };
                let mesh = controller.get_mesh_for_layer_mut(layer_proxy.index);
                mesh.set_vec4("color", color);
                mesh.visible = layer_proxy.visible;
                self.layer_proxies[layer_proxy.index] = layer_proxy.clone();
                controller.render();
                true
            }
            ViewerMsg::ToggleTheme => {
                self.is_dark_theme = !self.is_dark_theme;
                controller.apply_theme(if self.is_dark_theme { Theme::Dark } else { Theme::Light });
                for layer in controller.project().unwrap().layers() {
                    let proxy = &mut self.layer_proxies[layer.index() as usize];
                    proxy.color = rgb_to_hex(layer.color.x, layer.color.y, layer.color.z);
                    proxy.opacity = layer.color.w;
                }
                if let Some(window) = window() {
                    if let Some(storage) = window.local_storage().unwrap() {
                        let _ = storage.set_item("dark_theme", if self.is_dark_theme { "true" } else { "false" });
                    }
                }
                true
            }
        }
    }
}

// Helper function to fetch GDS file
async fn fetch_gds_file(id: &str) -> Result<Vec<u8>, wasm_bindgen::JsValue> {
    let opts = RequestInit::new();
    opts.set_method("GET");

    let url = format!("gds/{}.gds", id);

    let request = Request::new_with_str_and_init(&url, &opts)?;

    let window = web_sys::window().unwrap();
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
    let resp: Response = resp_value.dyn_into()?;

    // Get the response as an ArrayBuffer
    let array_buffer = JsFuture::from(resp.array_buffer()?).await?;
    let uint8_array = js_sys::Uint8Array::new(&array_buffer);
    let bytes = uint8_array.to_vec();

    Ok(bytes)
}
