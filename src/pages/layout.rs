use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::{closure::Closure, JsCast};
use web_sys::{window, HtmlCanvasElement, WebGl2RenderingContext};
use yew::prelude::*;
use yew_router::prelude::*;

use crate::{controller::Controller, gl_renderer::Renderer, gl_scene::Scene, Route};

type AnimationFrame = Rc<RefCell<Option<Closure<dyn FnMut()>>>>;

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
}

pub struct Layout {
    canvas_ref: NodeRef,
    controller: Option<Controller>,
    animation_frame: Option<Closure<dyn FnMut()>>,
}

impl Component for Layout {
    type Message = LayoutMsg;
    type Properties = LayoutProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            canvas_ref: NodeRef::default(),
            controller: None,
            animation_frame: None,
        }
    }

    fn destroy(&mut self, _ctx: &Context<Self>) {
        // Clean up the animation frame closure
        self.animation_frame = None;
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
                let controller = Controller::new(renderer, scene, width, height);
                self.controller = Some(controller);

                // Set up resize observer
                let canvas_clone = canvas.clone();
                let link = ctx.link().clone();
                let resize_observer = ResizeObserver::new(move |_entries, _observer| {
                    link.send_message(LayoutMsg::Render);
                });
                resize_observer.observe(&canvas_clone);

                // Set up animation frame loop
                let link = ctx.link().clone();
                let f: AnimationFrame = Rc::new(RefCell::new(None));
                let g = f.clone();

                *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
                    link.send_message(LayoutMsg::Tick);
                    // Request next frame
                    if let Some(window) = window() {
                        if let Some(closure) = f.borrow().as_ref() {
                            let _ =
                                window.request_animation_frame(closure.as_ref().unchecked_ref());
                        }
                    }
                }) as Box<dyn FnMut()>));

                // Store animation frame closure and start the loop
                if let Some(window) = window() {
                    if let Some(closure) = g.borrow().as_ref() {
                        let _ = window.request_animation_frame(closure.as_ref().unchecked_ref());
                    }
                }
                self.animation_frame = g.borrow_mut().take();

                // Initial render
                ctx.link().send_message(LayoutMsg::Render);
            }
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
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
                    controller.handle_mouse_wheel(x, y, delta);
                    controller.render();
                }
            }
        }
        false
    }
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
