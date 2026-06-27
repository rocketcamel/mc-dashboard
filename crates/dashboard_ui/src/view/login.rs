use yew::{AttrValue, Html, Properties, component, html};

use crate::components::display::{button::Button, input::Input};

#[derive(Properties, PartialEq)]
pub struct FieldProps {
    pub name: AttrValue,
    pub label: AttrValue,

    #[prop_or_default]
    pub placeholder: AttrValue,
    #[prop_or_default]
    pub field_type: AttrValue,
}

#[component(Field)]
fn field(props: &FieldProps) -> Html {
    html! {
        <div class="grid gap-2">
            <label for={props.name.clone()} class="text-xs/relaxed font-medium">
                { &props.label }
            </label>

            <Input placeholder={&props.placeholder} field_type={&props.field_type} />
        </div>
    }
}

#[component(Login)]
pub fn login() -> Html {
    html! {
        <div class="flex flex-col max-w-78 mx-auto mt-6 bg-card py-4 pb-0 gap-4">
            <div class="px-4">
                <p class="font-medium">{ "Login" }</p>
                <p class="text-xs text-muted-foreground">{ "Enter username below" }</p>
            </div>

            <div class="flex flex-col gap-6 px-4">
                <Field name="username" label="Username" placeholder="mrfartshit" />
                <Field name="password" label="Password" placeholder="Password" field_type="password" />
            </div>

            <div class="flex items-center bg-secondary p-4 border-t">
                <Button class="w-full py-0.5">{ "Login" }</Button>
            </div>
        </div>
    }
}
