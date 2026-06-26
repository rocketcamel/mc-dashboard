use yew::{Html, component, html};

use crate::{
    components::dropdown::{Dropdown, DropdownItem},
    icons::{LogOut, User},
};

#[component(Header)]
pub fn header() -> Html {
    let options = vec![DropdownItem {
        id: "1",
        label: "Logout",
        icon: Some(html! { <LogOut class="h-4 w-4"/> }),
    }];

    html! {
        <div class="flex max-w-6xl mx-auto justify-between p-4 items-center">
            <h1 class="text-xl font-bold">{ "mc-rocket-management" }</h1>

            <div class="rounded-full w-10 h-10 flex items-center justify-center bg-muted/50">
                <Dropdown {options} label="mrfartshit">
                    <User />
                </Dropdown>
            </div>

            </div>
    }
}
