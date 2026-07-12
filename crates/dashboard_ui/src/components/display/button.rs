use tw_merge::tw_merge;
use yew::{Callback, Children, Classes, Html, MouseEvent, Properties, component, html};

#[derive(Properties, PartialEq)]
pub struct ButtonProps {
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub button_type: Option<String>,

    #[prop_or_default]
    pub children: Children,
    #[prop_or_default]
    pub onclick: Callback<MouseEvent>,
    #[prop_or_default]
    pub disabled: bool,
}

#[component(Button)]
pub fn button(
    ButtonProps {
        class,
        button_type,
        children,
        onclick,
        disabled,
    }: &ButtonProps,
) -> Html {
    let button_classes = tw_merge!(
        "bg-primary text-primary-foreground font-medium text-xs/relaxed hover:bg-primary/80 transition-colors transition-transform duration-100 active:translate-y-px active:scale-[0.99] disabled:opacity-50 disabled:cursor-not-allowed disabled:pointer-events-none {}",
        class.to_string(),
    );

    html! {
        <button type={button_type.clone().unwrap_or("button".to_string())} disabled={*disabled} class={button_classes} {onclick}>
        { for children.iter() }
        </button>
    }
}
