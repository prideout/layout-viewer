use bevy_ecs::world::World;
use futures::StreamExt;
use gloo_timers::future::TimeoutFuture;
use js_sys::Promise;
use serde::Serialize;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use wasm_bindgen_futures::JsFuture;
use web_sys::window;
use web_sys::HtmlCanvasElement;
use web_sys::MouseEvent;
use web_sys::Request;
use web_sys::RequestInit;
use web_sys::Response;
use web_sys::WebGl2RenderingContext;
use web_sys::WheelEvent;
use yew::html::Scope;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::core::app_controller::AppController;
use crate::core::app_controller::Theme;
use crate::core::instancer::Instancer;
use crate::core::layer_proxy::LayerProxy;
use crate::core::loader::load_gds_into_world;
use crate::core::root_finder::RootFinder;
use crate::graphics::renderer::Renderer;
use crate::rsutils::resize_observer::ResizeObserver;
use crate::webui::app::Route;
use crate::webui::home_page::has_dropped_file;
use crate::webui::home_page::take_dropped_file;
use crate::webui::sidebar::Sidebar;
use crate::webui::toast::ToastContainer;
use crate::webui::toast::ToastManager;

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
    DoneFetching(Vec<u8>),
    SpawnLoader(Vec<u8>),
    SpawnInstancer(Box<World>),
    StashWorld(Box<World>),
    SetStatus(String),
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
    toast_manager: ToastManager,
    layer_proxies: Vec<LayerProxy>,
    is_dark_theme: bool,
    status: String,
    enabled: bool,
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
                storage
                    .get_item("dark_theme")
                    .unwrap_or(None)
                    .map(|s| s == "true")
                    .unwrap_or(false)
            } else {
                false
            }
        } else {
            false
        };

        if !has_dropped_file() && ctx.props().id == "dropped-file" {
            // No dropped file but on the dropped-file route, navigate back to home
            let navigator = ctx.link().navigator().unwrap();
            navigator.push(&Route::Home);
            log::info!("No dropped file found, redirecting to home page");
        }

        Self {
            canvas_ref,
            controller,
            toast_manager,
            layer_proxies,
            is_dark_theme,
            enabled: false,
            status: "Fetching GDS".to_string(),
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
                        <button class="floating-button" onclick={toggle_theme} disabled={!self.enabled}>
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

        let id = ctx.props().id.clone();
        let link = ctx.link().clone();

        if let Some((_name, content)) = take_dropped_file() {
            link.send_message(ViewerMsg::SpawnLoader(content));
        } else if id != "dropped-file" {
            download(link, id);
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
        })
        .unwrap();

        let gl: WebGl2RenderingContext = canvas
            .get_context_with_context_options("webgl2", &options)
            .unwrap()
            .unwrap()
            .dyn_into()
            .unwrap();

        // Create renderer with glow context
        let gl = glow::Context::from_webgl2_context(gl);
        let renderer = Renderer::new(gl);
        let width = canvas.client_width() as u32;
        let height = canvas.client_height() as u32;

        // Create controller
        let controller = AppController::new(renderer, width, height);
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
            ViewerMsg::DoneFetching(content) => {
                link.send_message(ViewerMsg::SpawnLoader(content));
                true
            }
            ViewerMsg::SpawnLoader(content) => {
                spawn_local(async move {
                    print_and_yield(&link, "Parsing GDS").await;

                    let mut progress_stream = load_gds_into_world(&content, World::new()).await;

                    print_and_yield(&link, "Loading GDS").await;

                    let mut progress_stream = std::pin::pin!(progress_stream);
                    let mut world = None;
                    while let Some(mut progress) = progress_stream.next().await {
                        let message = format!("{} {:.0}%", progress.phase, progress.percent);
                        print_and_yield(&link, &message).await;
                        world = progress.world.take();
                    }
                    let world = world.unwrap();
                    link.send_message(ViewerMsg::SpawnInstancer(Box::new(world)));
                });
                true
            }
            ViewerMsg::SpawnInstancer(world) => {
                spawn_local(async move {
                    let mut boxed_world = world;
                    let world = boxed_world.as_mut();
                    let mut root_finder = RootFinder::new(world);
                    let roots = root_finder.find_roots(world);

                    let message = format!("Found {} roots. Instancing...", roots.len());
                    print_and_yield(&link, &message).await;

                    let mut instancer = Instancer::new(world);
                    instancer.select_root(world, roots[0]);
                    link.send_message(ViewerMsg::StashWorld(boxed_world));
                });
                true
            }
            ViewerMsg::StashWorld(world) => {
                self.status.clear();

                let Some(controller) = &mut self.controller else {
                    spawn_local(async move {
                        print_and_yield(&link, "Waiting for app controller...").await;
                        link.send_message(ViewerMsg::StashWorld(world));
                    });
                    return true;
                };

                controller.set_world(*world);
                self.enabled = true;

                controller.apply_theme(if self.is_dark_theme {
                    Theme::Dark
                } else {
                    Theme::Light
                });

                self.layer_proxies = controller.create_layer_proxies();
                true
            }
            ViewerMsg::SetStatus(status) => {
                self.status = status;
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
                controller.update_layer(layer_proxy);
                self.layer_proxies = controller.create_layer_proxies();
                controller.render();
                true
            }
            ViewerMsg::ToggleTheme => {
                self.is_dark_theme = !self.is_dark_theme;
                controller.apply_theme(if self.is_dark_theme {
                    Theme::Dark
                } else {
                    Theme::Light
                });
                if let Some(window) = window() {
                    if let Some(storage) = window.local_storage().unwrap() {
                        let _ = storage.set_item(
                            "dark_theme",
                            if self.is_dark_theme { "true" } else { "false" },
                        );
                    }
                }
                true
            }
        }
    }
}

// Helper function to fetch GDS file
async fn fetch_gds_file(filename: &str) -> Result<Vec<u8>, wasm_bindgen::JsValue> {
    let opts = RequestInit::new();
    opts.set_method("GET");

    let url = format!("gds/{}.gds", filename);

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

fn download(link: Scope<ViewerPage>, filename: String) {
    wasm_bindgen_futures::spawn_local(async move {
        match fetch_gds_file(&filename).await {
            Ok(bytes) => {
                link.send_message(ViewerMsg::DoneFetching(bytes));
            }
            Err(e) => {
                log::error!("Failed to fetch GDS file: {:?}", e);
            }
        }
    });
}

async fn print_and_yield(link: &Scope<ViewerPage>, status: &str) {
    link.send_message(ViewerMsg::SetStatus(status.to_string()));
    TimeoutFuture::new(0).await;
}

// NOTE: This is the more modern API than TimeoutFuture, but it does not seem to
// give enough time for the browser to re-render the DOM.
#[allow(dead_code)]
async fn yield_to_browser() -> Result<JsValue, JsValue> {
    JsFuture::from(Promise::new(&mut |resolve, _| {
        web_sys::window().unwrap().queue_microtask(&resolve);
    }))
    .await
}
