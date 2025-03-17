use wasm_bindgen::{closure::Closure, JsCast};
use wasm_bindgen_futures::JsFuture;
use web_sys::{window, HtmlCanvasElement, Request, RequestInit, Response, WebGl2RenderingContext};
use yew::prelude::*;
use yew_router::prelude::*;

use crate::{
    controller::Controller, gl_renderer::Renderer, gl_scene::Scene, populate_scene, Project, Route,
};

#[derive(Properties, PartialEq)]
pub struct LayoutProps {
    pub id: String,
}

pub enum LayoutMsg {
    Render,
    MousePress(u32, u32),
    MouseRelease,
    MouseMove(u32, u32),
    MouseWheel(u32, u32, f32),
    Tick,
    GdsLoaded(Box<Project>),
}

pub struct Layout {
    canvas_ref: NodeRef,
    controller: Option<Controller>,
}

impl Component for Layout {
    type Message = LayoutMsg;
    type Properties = LayoutProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            canvas_ref: NodeRef::default(),
            controller: None,
        }
    }

    fn destroy(&mut self, _ctx: &Context<Self>) {
        // No cleanup needed anymore
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let onmousedown = ctx.link().callback(|e: MouseEvent| {
            let x = e.offset_x() as u32;
            let y = e.offset_y() as u32;
            LayoutMsg::MousePress(x, y)
        });

        let onmouseup = ctx.link().callback(|_| LayoutMsg::MouseRelease);

        let onmousemove = ctx.link().callback(|e: MouseEvent| {
            let x = e.offset_x() as u32;
            let y = e.offset_y() as u32;
            LayoutMsg::MouseMove(x, y)
        });

        let onwheel = ctx.link().callback(|e: WheelEvent| {
            e.prevent_default();
            let x = e.offset_x() as u32;
            let y = e.offset_y() as u32;
            let delta = e.delta_y() as f32;
            LayoutMsg::MouseWheel(x, y, delta)
        });

        html! {
            <div class="layout-container">
                <canvas
                    ref={self.canvas_ref.clone()}
                    class="layout-canvas"
                    {onmousedown}
                    {onmouseup}
                    {onmousemove}
                    {onwheel}
                />
                <div class="floating-buttons">
                    <Link<Route> to={Route::Home} classes="floating-button">
                        <i class="fas fa-home"></i>
                    </Link<Route>>
                    <a href="https://github.com/prideout/layout-viewer"
                       class="floating-button"
                       target="_blank">
                        <i class="fab fa-github"></i>
                    </a>
                </div>
            </div>
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            // Start GDS file fetch
            let id = ctx.props().id.clone();
            let link = ctx.link().clone();
            wasm_bindgen_futures::spawn_local(async move {
                match fetch_gds_file(&id).await {
                    Ok(bytes) => {
                        log::info!("Parsing {} bytes from GDS file", bytes.len());
                        match Project::from_bytes(&bytes) {
                            Ok(project) => {
                                let stats = project.stats();
                                log::info!("Loaded GDS file for project {}", id);
                                log::info!("Number of structs: {}", stats.struct_count);
                                log::info!("Number of polygons: {}", stats.polygon_count);
                                log::info!("Number of paths: {}", stats.path_count);
                                log::info!(
                                    "Number of layers: {}",
                                    (project.highest_layer() + 1) as usize
                                );
                                link.send_message(LayoutMsg::GdsLoaded(Box::new(project)));
                            }
                            Err(_) => {
                                log::error!("Failed to parse GDS file.");
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to fetch GDS file: {:?}", e);
                    }
                }
            });

            // Get canvas and create WebGL context
            if let Some(canvas) = self.canvas_ref.cast::<HtmlCanvasElement>() {
                let gl: WebGl2RenderingContext = canvas
                    .get_context("webgl2")
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
                log::info!("Creating controller at {}x{}", width, height);
                let controller = Controller::new(renderer, scene, width, height);
                self.controller = Some(controller);

                // Set up resize observer
                let canvas_clone = canvas.clone();
                let link = ctx.link().clone();
                let resize_observer = ResizeObserver::new(move |_entries, _observer| {
                    link.send_message(LayoutMsg::Render);
                });
                resize_observer.observe(&canvas_clone);

                ctx.link().send_message(LayoutMsg::Tick);
                ctx.link().send_message(LayoutMsg::Render);
            }
        }
    }

    fn update(&mut self, context: &Context<Self>, msg: Self::Message) -> bool {
        let link = context.link().clone();
        if let Some(controller) = &mut self.controller {
            match msg {
                LayoutMsg::Render => {
                    if let Some(canvas) = self.canvas_ref.cast::<HtmlCanvasElement>() {
                        let width = canvas.client_width() as u32;
                        let height = canvas.client_height() as u32;
                        canvas.set_width(width);
                        canvas.set_height(height);
                        controller.resize(width, height);
                        controller.render();
                    }
                }
                LayoutMsg::Tick => {
                    controller.tick();
                    let closure = Closure::wrap(Box::new(move || {
                        link.send_message(LayoutMsg::Tick);
                    }) as Box<dyn FnMut()>);
                    if let Some(window) = window() {
                        let _ = window.request_animation_frame(closure.as_ref().unchecked_ref());
                    }
                    closure.forget();
                }
                LayoutMsg::MousePress(x, y) => {
                    controller.handle_mouse_press(x, y);
                    controller.render();
                }
                LayoutMsg::MouseRelease => {
                    controller.handle_mouse_release();
                    controller.render();
                }
                LayoutMsg::MouseMove(x, y) => {
                    controller.handle_mouse_move(x, y);
                    controller.render();
                }
                LayoutMsg::MouseWheel(x, y, delta) => {
                    controller.handle_mouse_wheel(x, y, -delta);
                    controller.render();
                }
                LayoutMsg::GdsLoaded(project) => {
                    if let Some(controller) = &mut self.controller {
                        populate_scene(project.render_layers(), controller.scene());
                        controller.render();
                        log::info!("Scene populated");
                    } else {
                        // This is a potential race condition, but it's highly unlikely to happen.
                        log::error!("Controller not ready");
                    }
                }
            }
        }
        false
    }
}

// Helper function to fetch GDS file
async fn fetch_gds_file(id: &str) -> Result<Vec<u8>, wasm_bindgen::JsValue> {
    let opts = RequestInit::new();
    opts.set_method("GET");

    let url = format!("../gds/{}.gds", id);
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

// Helper struct for resize observer
struct ResizeObserver(web_sys::ResizeObserver);

impl ResizeObserver {
    fn new<F>(callback: F) -> Self
    where
        F: FnMut(Vec<web_sys::ResizeObserverEntry>, web_sys::ResizeObserver) + 'static,
    {
        let callback = Closure::wrap(Box::new(callback) as Box<dyn FnMut(_, _)>);
        let observer = web_sys::ResizeObserver::new(callback.as_ref().unchecked_ref()).unwrap();
        callback.forget();
        Self(observer)
    }

    fn observe(&self, target: &web_sys::Element) {
        self.0.observe(target);
    }
}
