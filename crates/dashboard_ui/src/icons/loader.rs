use yew::{Classes, Html, Properties, component, html};

#[derive(Properties, PartialEq)]
pub struct LoaderProps {
    pub class: Classes,
}

#[component(Loader)]
pub fn loader(LoaderProps { class }: &LoaderProps) -> Html {
    html! {
        <svg {class} xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-loader-circle-icon lucide-loader-circle"><path d="M21 12a9 9 0 1 1-6.219-8.56"/></svg>
    }
}
