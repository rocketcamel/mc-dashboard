use yew::{Callback, Children, Classes, Html, MouseEvent, Properties, classes, component, html};

#[derive(Properties, PartialEq)]
pub struct ButtonProps {
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub children: Children,
    #[prop_or_default]
    pub onclick: Callback<MouseEvent>,
}

#[component(Button)]
pub fn button(
    ButtonProps {
        class,
        children,
        onclick,
    }: &ButtonProps,
) -> Html {
    let button_classes = classes!(
        "bg-primary",
        "text-primary-foreground",
        "font-medium",
        "text-xs/relaxed",
        "hover:bg-primary/80",
        "transition-colors",
        "transition-transform",
        "duration-100",
        "active:translate-y-px",
        "active:scale-[0.99]",
        class.clone(),
    );

    html! {
        <button type="button" disabled={true} class={button_classes} {onclick}>
        { for children.iter() }
        </button>
    }
}
