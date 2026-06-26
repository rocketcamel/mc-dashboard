mod components;
mod icons;
mod view;

use yew::{Children, Html, Properties, component, html};
use yew_router::{BrowserRouter, Routable, Switch};

use crate::components::header::Header;

#[derive(Routable, Clone, PartialEq)]
enum Route {
    #[at("/")]
    Index,
    #[at("/404")]
    NotFound,
}

#[derive(Properties, PartialEq)]
struct LayoutProps {
    pub children: Children,
}

#[component(Index)]
fn index() -> Html {
    html! {
        <div class="flex flex-col items-center max-w-334 mx-auto mt-6">
          <h1 class="text-2xl font-semibold border-b-2 pb-1 px-6">
            { "Management" }
          </h1>
          <div class="grid grid-cols-1 lg:grid-cols-3 gap-6 p-6 mb-10">
          </div>
        </div>
    }
}

#[component(Layout)]
fn layout(props: &LayoutProps) -> Html {
    html! {
        <div id="root">
            <Header />
            { props.children.clone() }
        </div>
    }
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::Index => html! { <Index /> },
        Route::NotFound => html! { <p>{ "Not Found" }</p> },
    }
}

#[component]
fn App() -> Html {
    html! {
        <BrowserRouter>
            <Layout>
                <Switch<Route> render={switch} />
            </Layout>
        </BrowserRouter>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
