use web_sys::HtmlInputElement;
use yew::prelude::*;

#[derive(Clone, PartialEq)]
pub struct LayerProxy {
    pub index: usize,
    pub visible: bool,
    pub opacity: f32,
    pub color: String,
}

#[derive(Properties, PartialEq)]
pub struct SidebarProps {
    pub layers: Vec<LayerProxy>,
    pub update_layer: Callback<LayerProxy>,
}

pub enum SidebarMsg {
    HideAll,
    ShowAll,
    ToggleLayer(usize),
    UpdateOpacity(usize, f32),
    UpdateColor(usize, String),
}

pub struct Sidebar;

impl Component for Sidebar {
    type Message = SidebarMsg;
    type Properties = SidebarProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let hide_all = ctx.link().callback(|_| SidebarMsg::HideAll);
        let show_all = ctx.link().callback(|_| SidebarMsg::ShowAll);

        html! {
            <div class="sidebar">
                <div class="sidebar-header">
                    <button onclick={hide_all}>{"Hide All"}</button>
                    <button onclick={show_all}>{"Show All"}</button>
                </div>
                <div class="layer-list">
                    {ctx.props().layers.iter().map(|layer| {
                        let index = layer.index;
                        let toggle_layer = ctx.link().callback(move |_| SidebarMsg::ToggleLayer(index));
                        let update_opacity = ctx.link().callback(move |e: InputEvent| {
                            let input: HtmlInputElement = e.target_unchecked_into();
                            let opacity = input.value().parse::<f32>().unwrap_or(1.0);
                            SidebarMsg::UpdateOpacity(index, opacity)
                        });
                        let update_color = ctx.link().callback(move |e: InputEvent| {
                            let input: HtmlInputElement = e.target_unchecked_into();
                            SidebarMsg::UpdateColor(index, input.value())
                        });
                        let prevent_toggle = |e: MouseEvent| {
                            e.stop_propagation();
                        };

                        html! {
                            <div
                                class="layer-item"
                                key={layer.index}
                                onclick={toggle_layer}
                                style={format!(
                                    "background-color: {}",
                                    if layer.visible { "#3d3d3d" } else { "#2d2d2d" }
                                )}
                            >
                                <i class={format!("fas fa-eye{}", if layer.visible { "" } else { "-slash" })}></i>
                                <div class="color-picker-container" onclick={prevent_toggle}>
                                    <span class="layer-color" style={format!("background-color: {}", layer.color)}></span>
                                    <input
                                        type="color"
                                        value={layer.color.clone()}
                                        oninput={update_color}
                                        class="color-picker"
                                    />
                                </div>
                                <span class="layer-index">{format!("Layer {}", layer.index)}</span>
                                <input
                                    type="range"
                                    min="0"
                                    max="1"
                                    step="0.01"
                                    value={layer.opacity.to_string()}
                                    oninput={update_opacity}
                                    onclick={prevent_toggle}
                                />
                            </div>
                        }
                    }).collect::<Html>()}
                </div>
            </div>
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let find_layer_proxy = |index: usize| {
            ctx.props().layers.iter().find(|layer| layer.index == index)
        };
        match msg {
            SidebarMsg::HideAll => {
                for layer in &ctx.props().layers {
                    let mut layer = layer.clone();
                    layer.visible = false;
                    ctx.props().update_layer.emit(layer);
                }
                true
            }
            SidebarMsg::ShowAll => {
                for layer in &ctx.props().layers {
                    let mut layer = layer.clone();
                    layer.visible = true;
                    ctx.props().update_layer.emit(layer);
                }
                true
            }
            SidebarMsg::ToggleLayer(index) => {
                if let Some(layer) = find_layer_proxy(index) {
                    let mut layer = layer.clone();
                    layer.visible = !layer.visible;
                    ctx.props().update_layer.emit(layer.clone());
                }
                true
            }
            SidebarMsg::UpdateOpacity(index, opacity) => {
                if let Some(layer) = find_layer_proxy(index) {
                    let mut layer = layer.clone();
                    layer.opacity = opacity;
                    ctx.props().update_layer.emit(layer.clone());
                }
                true
            }
            SidebarMsg::UpdateColor(index, color) => {
                if let Some(layer) = find_layer_proxy(index) {
                    let mut layer = layer.clone();
                    layer.color = color.clone();
                    ctx.props().update_layer.emit(layer.clone());
                }
                true
            }
        }
    }
}
