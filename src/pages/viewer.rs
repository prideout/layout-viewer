use wasm_bindgen::{closure::Closure, JsCast};
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    window, HtmlCanvasElement, MouseEvent, Request, RequestInit, Response, WebGl2RenderingContext,
    WheelEvent,
};
use yew::prelude::*;
use yew_router::prelude::*;

use crate::{
    colors::{hex_to_rgb, rgb_to_hex},
    components::{
        sidebar::{LayerProxy, Sidebar},
        toast::{ToastContainer, ToastManager},
    },
    controller::Controller,
    gl_mesh::MeshId,
    gl_renderer::Renderer,
    gl_scene::Scene,
    pages::home::take_dropped_file,
    resize_observer::ResizeObserver,
    Project, Route,
};

#[derive(Properties, PartialEq)]
pub struct ViewerProps {
    pub id: String,
}

pub enum ViewerMsg {
    MousePress(u32, u32),
    MouseRelease,
    MouseMove(u32, u32),
    MouseWheel(u32, u32, f32),
    GdsLoaded(Box<Project>),
    ParsingGds,
    Render,
    Tick,
    RemoveToast(usize),
    UpdateLayer(LayerProxy),
}

pub struct ViewerPage {
    canvas_ref: NodeRef,
    controller: Option<Controller>,
    status: String,
    toast_manager: ToastManager,
    layer_proxies: Vec<LayerProxy>,
}

impl Component for ViewerPage {
    type Message = ViewerMsg;
    type Properties = ViewerProps;

    fn create(ctx: &Context<Self>) -> Self {
        let canvas_ref = NodeRef::default();
        let controller = None;
        let status = "Downloading GDS...".to_string();
        let toast_manager = ToastManager::new();
        let layer_proxies = Vec::new();

        // Check for dropped file
        if let Some((_name, content)) = take_dropped_file() {
            let link = ctx.link().clone();
            wasm_bindgen_futures::spawn_local(async move {
                link.send_message(ViewerMsg::ParsingGds);
                match Project::from_bytes(&content) {
                    Ok(mut project) => {
                        project.update_layers();
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
            status,
            toast_manager,
            layer_proxies,
        }
    }

    fn destroy(&mut self, _ctx: &Context<Self>) {
        log::info!("Destroying controller...");
        self.controller = None;
        log::info!("Done destroying controller.");
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let onmousedown = ctx.link().callback(|e: MouseEvent| {
            let x = e.offset_x() as u32;
            let y = e.offset_y() as u32;
            ViewerMsg::MousePress(x, y)
        });

        let onmouseup = ctx.link().callback(|_| ViewerMsg::MouseRelease);

        let onmousemove = ctx.link().callback(|e: MouseEvent| {
            let x = e.offset_x() as u32;
            let y = e.offset_y() as u32;
            ViewerMsg::MouseMove(x, y)
        });

        let onwheel = ctx.link().callback(|e: WheelEvent| {
            e.prevent_default();
            let x = e.offset_x() as u32;
            let y = e.offset_y() as u32;
            let delta = e.delta_y() as f32;
            ViewerMsg::MouseWheel(x, y, delta)
        });

        let on_remove_toast = ctx.link().callback(ViewerMsg::RemoveToast);
        let update_layer = ctx.link().callback(ViewerMsg::UpdateLayer);

        html! {
            <>
                <div class="viewer-container">
                    <canvas
                        class="viewer-canvas"
                        ref={self.canvas_ref.clone()}
                        onmousedown={onmousedown}
                        onmouseup={onmouseup}
                        onmousemove={onmousemove}
                        onwheel={onwheel}
                    />
                    <div class="floating-buttons">
                        <Link<Route> to={Route::Home} classes="floating-button">
                            <i class="fas fa-arrow-left fa-lg"></i>
                        </Link<Route>>
                        <span class="status-text">{self.status.clone()}</span>
                    </div>
                </div>
                <Sidebar layers={self.layer_proxies.clone()} update_layer={update_layer} />
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
                        link.send_message(ViewerMsg::ParsingGds);
                        let Ok(project) = Project::from_bytes(&bytes) else {
                            log::error!("Failed to parse fetched GDS.");
                            return;
                        };
                        let stats = project.stats();
                        log::info!("Number of structs: {}", stats.struct_count);
                        log::info!("Number of polygons: {}", stats.polygon_count);
                        log::info!("Number of paths: {}", stats.path_count);
                        log::info!(
                            "Number of layers: {}",
                            (project.highest_layer() + 1) as usize
                        );
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
        let controller = Controller::new(renderer, scene, width, height);
        self.controller = Some(controller);

        // Set up resize observer
        let canvas_clone = canvas.clone();
        let link = ctx.link().clone();
        let resize_observer = ResizeObserver::new(move |_entries, _observer| {
            link.send_message(ViewerMsg::Render);
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
            ViewerMsg::Render => {
                if let Some(canvas) = self.canvas_ref.cast::<HtmlCanvasElement>() {
                    let width = canvas.client_width() as u32;
                    let height = canvas.client_height() as u32;
                    canvas.set_width(width);
                    canvas.set_height(height);
                    controller.resize(width, height);
                    controller.render();
                }
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
                controller.render();
                false
            }
            ViewerMsg::MouseRelease => {
                controller.handle_mouse_release();
                controller.render();
                false
            }
            ViewerMsg::MouseMove(x, y) => {
                controller.handle_mouse_move(x, y);
                controller.render();
                false
            }
            ViewerMsg::MouseWheel(x, y, delta) => {
                controller.handle_mouse_wheel(x, y, -delta);
                controller.render();
                false
            }
            ViewerMsg::GdsLoaded(project) => {
                let Some(controller) = &mut self.controller else {
                    log::error!("Controller not ready");
                    return false;
                };
                controller.set_project(*project);
                self.status.clear();
                self.toast_manager
                    .show("Zoom and pan like a map".to_string());

                // Update layer proxies
                if let Some(project) = controller.project() {
                    self.layer_proxies = project
                        .layers()
                        .iter()
                        .enumerate()
                        .filter_map(|(index, layer)| {
                            if layer.polygons.is_empty() {
                                return None;
                            }
                            let color = if index == project.layers().len() - 1 {
                                // Make highest layer white
                                rgb_to_hex(1.0, 1.0, 1.0)
                            } else {
                                rgb_to_hex(layer.color.x, layer.color.y, layer.color.z)
                            };
                            Some(LayerProxy {
                                index,
                                visible: true,
                                opacity: layer.color.w,
                                color,
                            })
                        })
                        .collect();
                }

                controller.render();
                true
            }
            ViewerMsg::ParsingGds => {
                self.status = "Parsing GDS...".to_string();
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

                    if let Some((r, g, b)) = hex_to_rgb(&layer_proxy.color) {
                        layer.color.w = layer_proxy.opacity;
                        layer.color.x = r;
                        layer.color.y = g;
                        layer.color.z = b;
                    }
                    layer.color
                };
                let mesh_id = MeshId(1 + layer_proxy.index);
                let mesh = controller.scene().get_mesh_mut(&mesh_id).unwrap();
                mesh.set_vec4("color", color);
                mesh.visible = layer_proxy.visible;
                if let Some(layer) = self
                    .layer_proxies
                    .iter_mut()
                    .find(|layer| layer.index == layer_proxy.index)
                {
                    *layer = layer_proxy.clone();
                }
                controller.render();
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
