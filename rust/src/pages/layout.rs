use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub id: String,
}

#[function_component(Layout)]
pub fn layout(props: &Props) -> Html {
    html! {
        <div>
            <h1>{format!("Layout: {}", props.id)}</h1>
            // Layout viewer will be implemented here
        </div>
    }
} 