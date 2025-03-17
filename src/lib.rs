mod bounds;
mod cells;
mod gl_backend;
mod id_map;
mod layer;
mod project;
mod string_interner;
mod svg_backend;

pub mod colors;
pub mod controller;
pub mod gl_camera;
pub mod gl_geometry;
pub mod gl_material;
pub mod gl_mesh;
pub mod gl_renderer;
pub mod gl_scene;
pub mod gl_viewport;

#[cfg(not(target_arch = "wasm32"))]
pub use gl_window::spawn_window;

#[cfg(not(target_arch = "wasm32"))]
pub mod gl_window;

pub use gl_scene::Scene;
pub use project::Project;

pub use gl_backend::populate_scene;
pub use svg_backend::generate_svg;

#[cfg(target_arch = "wasm32")]
mod pages;

#[cfg(target_arch = "wasm32")]
mod resize_observer;

#[cfg(target_arch = "wasm32")]
mod components;

#[cfg(target_arch = "wasm32")]
use pages::{home::HomePage, viewer::ViewerPage};

#[cfg(target_arch = "wasm32")]
use yew::prelude::*;

#[cfg(target_arch = "wasm32")]
use yew_router::prelude::*;

#[cfg(target_arch = "wasm32")]
#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[at("/")]
    Home,
    #[at("/:id")]
    Viewer { id: String },
}

#[cfg(target_arch = "wasm32")]
fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => html! { <HomePage /> },
        Route::Viewer { id } => html! { <ViewerPage id={id} /> },
    }
}

#[cfg(target_arch = "wasm32")]
#[function_component(App)]
pub fn app() -> Html {
    html! {
        <HashRouter>
            <Switch<Route> render={switch} />
        </HashRouter>
    }
}

#[cfg(target_arch = "wasm32")]
pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

/// Returns a timestamp in milliseconds.
#[cfg(target_arch = "wasm32")]
#[macro_export]
macro_rules! performance_now {
    () => {
        web_sys::window()
            .and_then(|w| w.performance().and_then(|f| Some(f.now())))
            .unwrap_or(0.0)
    };
}
