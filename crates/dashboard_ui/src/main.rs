mod components;
mod icons;
mod net;
mod styles;
mod view;

use tracing::Level;
use tracing_subscriber::filter::Targets;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use yew::html::ChildrenProps;
use yew::{
    Children, Html, Properties, component, html, platform::spawn_local, use_effect_with, use_state,
};

use yew_router::{BrowserRouter, Routable, Switch, hooks::use_navigator};

use crate::view::Index;
use crate::{
    components::header::Header,
    net::{AuthStatus, get_auth_status},
    view::login::Login,
};

#[derive(Routable, Clone, PartialEq)]
enum Route {
    #[at("/")]
    Index,
    #[at("/login")]
    Login,
    #[at("/404")]
    #[not_found]
    NotFound,
}

#[derive(Properties, PartialEq)]
struct LayoutProps {
    pub children: Children,
}

#[derive(Properties, PartialEq)]
struct ProtectedProps {
    pub children: Children,
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

#[component(Protected)]
fn protected(props: &ProtectedProps) -> Html {
    let auth_ok = use_state(|| false);
    let navigator = use_navigator().unwrap();

    use_effect_with((), {
        let navigator = navigator.clone();
        let auth_ok = auth_ok.clone();

        move |_| {
            spawn_local(async move {
                let status = get_auth_status().await;

                match status {
                    Ok(AuthStatus::Authenticated(_)) => auth_ok.set(true),
                    Ok(AuthStatus::Unauthenticated) | Err(_) => navigator.replace(&Route::Login),
                }
            });
        }
    });

    if *auth_ok {
        html! { { for props.children.iter() } }
    } else {
        html! {}
    }
}

#[component(Public)]
fn public(props: &ChildrenProps) -> Html {
    let render = use_state(|| false);
    let navigator = use_navigator().unwrap();

    use_effect_with((), {
        let navigator = navigator.clone();
        let render = render.clone();

        move |_| {
            spawn_local(async move {
                let status = get_auth_status().await;

                match status {
                    Ok(AuthStatus::Authenticated(_)) => navigator.replace(&Route::Index),
                    Ok(AuthStatus::Unauthenticated) | Err(_) => render.set(true),
                }
            });
        }
    });

    if *render {
        html! { { props.children.clone() } }
    } else {
        html! {}
    }
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::Index => html! {
            <Protected>
                <Layout>
                    <Index />
                </Layout>
            </Protected>
        },
        Route::Login => html! { <Public><Login /></Public> },
        Route::NotFound => html! { <p>{ "Not Found" }</p> },
    }
}

#[component]
fn App() -> Html {
    html! {
        <BrowserRouter>
            <Switch<Route> render={switch} />
        </BrowserRouter>
    }
}

fn main() {
    let tracing_wasm_layer = tracing_wasm::WASMLayer::new(tracing_wasm::WASMLayerConfig::default());
    let filter = Targets::new().with_default(Level::INFO);

    tracing_subscriber::registry()
        .with(tracing_wasm_layer)
        .with(filter)
        .init();

    yew::Renderer::<App>::new().render();
}
