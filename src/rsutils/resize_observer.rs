use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::Element;
use web_sys::ResizeObserver as WebResizeObserver;
use web_sys::ResizeObserverEntry;

pub struct ResizeObserver(WebResizeObserver);

impl ResizeObserver {
    pub fn new<F>(callback: F) -> Self
    where
        F: FnMut(Vec<ResizeObserverEntry>, WebResizeObserver) + 'static,
    {
        let callback = Closure::wrap(Box::new(callback) as Box<dyn FnMut(_, _)>);
        let observer = WebResizeObserver::new(callback.as_ref().unchecked_ref()).unwrap();
        callback.forget();
        Self(observer)
    }

    pub fn observe(&self, target: &Element) {
        self.0.observe(target);
    }
}
