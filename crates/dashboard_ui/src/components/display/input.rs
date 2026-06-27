use web_sys::HtmlInputElement;
use yew::{AttrValue, Callback, Html, InputEvent, Properties, TargetCast, component, html};

#[derive(Properties, PartialEq)]
pub struct InputProps {
    #[prop_or_default]
    pub on_update: Callback<String>,
    #[prop_or_default]
    pub invalid: bool,

    #[prop_or_default]
    pub placeholder: AttrValue,
    #[prop_or_default]
    pub field_type: AttrValue,
}

#[component(Input)]
pub fn input(props: &InputProps) -> Html {
    let on_change = {
        let on_update = props.on_update.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            on_update.emit(input.value())
        })
    };

    let input_classes = format!(
        "w-full text-xs/relaxed h-7 border bg-input/20 px-2 py-0.5 rounded-md outline-none transition-colors {}",
        if props.invalid {
            "border-red-500 focus-visible:ring-2 focus-visible:ring-red-500/30"
        } else {
            "border-input focus-visible:border-ring focus-visible:ring-2 focus-visible:ring-ring/30"
        }
    );

    html! {
        <input class={input_classes}
               aria-invalid={props.invalid.to_string()}
               placeholder={&props.placeholder}
               type={&props.field_type}
               oninput={on_change}
        />
    }
}
