use yew::{Classes, Html, Properties, component, html};

#[derive(Properties, PartialEq)]
pub struct Props {
    #[prop_or_default]
    pub class: Classes,
}

#[component(LogOut)]
pub fn log_out(Props { class }: &Props) -> Html {
    html! {
        <svg {class} xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-log-out-icon lucide-log-out"><path d="m16 17 5-5-5-5"/><path d="M21 12H9"/><path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4"/></svg>
    }
}

#[component(User)]
pub fn user(Props { class }: &Props) -> Html {
    html! {
        <svg {class} xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-user-icon lucide-user"><path d="M19 21v-2a4 4 0 0 0-4-4H9a4 4 0 0 0-4 4v2"/><circle cx="12" cy="7" r="4"/></svg>
    }
}
