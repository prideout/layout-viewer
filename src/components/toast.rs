use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::window;
use yew::prelude::*;

#[derive(Clone, PartialEq)]
pub struct Toast {
    pub id: usize,
    pub message: String,
}

#[derive(Properties, PartialEq)]
pub struct ToastProps {
    pub toasts: Vec<Toast>,
    pub on_remove: Callback<usize>,
}

#[function_component(ToastContainer)]
pub fn toast_container(props: &ToastProps) -> Html {
    let on_remove = props.on_remove.clone();

    // Set up auto-removal for each toast
    for toast in props.toasts.iter() {
        let on_remove = on_remove.clone();
        let id = toast.id;

        let window = window().unwrap();
        let timeout = window
            .set_timeout_with_callback_and_timeout_and_arguments_0(
                Closure::wrap(Box::new(move || {
                    on_remove.emit(id);
                }) as Box<dyn FnMut()>)
                .into_js_value()
                .unchecked_ref(),
                3000,
            )
            .unwrap();

        // Clean up the timeout when the component is destroyed
        let window_clone = window.clone();
        let cleanup = Closure::wrap(Box::new(move || {
            window_clone.clear_timeout_with_handle(timeout);
        }) as Box<dyn FnMut()>);

        // Store the cleanup closure to be called when the component is destroyed
        let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
            cleanup.into_js_value().unchecked_ref(),
            3000,
        );
    }

    html! {
        <div class="toast-container">
            {for props.toasts.iter().map(|toast| {
                html! {
                    <div key={toast.id} class="toast">
                        {&toast.message}
                    </div>
                }
            })}
        </div>
    }
}

pub struct ToastManager {
    toasts: Vec<Toast>,
    next_id: usize,
}

impl ToastManager {
    pub fn new() -> Self {
        Self {
            toasts: Vec::new(),
            next_id: 0,
        }
    }

    pub fn show(&mut self, message: String) {
        let id = self.next_id;
        self.next_id += 1;
        self.toasts.push(Toast { id, message });
    }

    pub fn remove(&mut self, id: usize) {
        self.toasts.retain(|t| t.id != id);
    }

    pub fn toasts(&self) -> &[Toast] {
        &self.toasts
    }
}
