use yew::{Callback, Html, component, html, platform::spawn_local};
use yew_router::hooks::use_navigator;

use crate::{
    Route,
    components::dropdown::{Dropdown, DropdownItem},
    icons::{LogOut, User},
    net::logout as logout_mutation,
};

#[component(Header)]
pub fn header() -> Html {
    let navigator = use_navigator().unwrap();

    let logout = {
        let navigator = navigator.clone();
        Callback::from(move |_| {
            let navigator = navigator.clone();
            spawn_local(async move {
                logout_mutation().await.ok();
                navigator.replace(&Route::Login);
            });
        })
    };

    let options = vec![DropdownItem {
        id: "logout",
        content: html! {
            <div class="flex justify-between">
                <span> { "Logout" }</span>
                <span class="inline-flex ml-auto items-center"><LogOut  class="h-4 w-4" /></span>
            </div>
        },
    }];

    let update_selected = Callback::from(move |id: &'static str| match id {
        "logout" => logout.emit(()),
        _ => {}
    });

    html! {
        <div class="flex max-w-6xl mx-auto justify-between p-4 items-center">
            <h1 class="text-xl font-bold">{ "mc-rocket-management" }</h1>

            <div class="rounded-full w-10 h-10 flex items-center justify-center bg-muted/50">
                <Dropdown<&'static str> {update_selected} {options} label="mrfartshit">
                    <User />
                </Dropdown<&'static str>>
            </div>

            </div>
    }
}
