use yew::{Children, Classes, Html, Properties, classes, component, html};

#[derive(Properties, PartialEq)]
pub struct ButtonProps {
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub children: Children,
}

#[component(Button)]
pub fn button(ButtonProps { class, children }: &ButtonProps) -> Html {
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
        <button type="button" class={button_classes}>
        { for children.iter() }
        </button>
    }
}
