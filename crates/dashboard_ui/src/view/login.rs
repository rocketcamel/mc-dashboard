use thiserror_ext::AsReport;
use yew::{
    AttrValue, Callback, Html, Properties, Reducible, SubmitEvent, component, html,
    platform::spawn_local, use_reducer,
};
use yew_router::hooks::use_navigator;

use crate::{
    Route,
    components::display::{button::Button, input::Input},
    icons::Loader,
    net::{
        LoginStatus::{InvalidCredentials, Success},
        login as login_mutation,
    },
};

#[derive(Clone, PartialEq, Default)]
pub struct LoginState {
    pub username: String,
    pub password: String,
    pub touched_username: bool,
    pub touched_password: bool,

    pub submit_attempted: bool,
    pub submitting: bool,
    pub error: Option<String>,
}

pub enum LoginAction {
    SetUsername(String),
    SetPassword(String),
    BlurUsername,
    BlurPassword,

    SubmitAttempt,
    SubmitStarted,
    SubmitFailed(String),
    SubmitDone,
}

impl LoginState {
    pub fn username_error(&self) -> Option<&'static str> {
        self.username
            .trim()
            .is_empty()
            .then_some("username field is required")
    }

    pub fn password_error(&self) -> Option<&'static str> {
        self.password
            .trim()
            .is_empty()
            .then_some("password field is required")
    }

    pub fn show_username_error(&self) -> bool {
        (self.touched_username || self.submit_attempted) && self.username_error().is_some()
    }

    pub fn show_password_error(&self) -> bool {
        (self.touched_password || self.submit_attempted) && self.password_error().is_some()
    }

    pub fn can_submit(&self) -> bool {
        self.username_error().is_none() && self.password_error().is_none() && !self.submitting
    }
}

impl Reducible for LoginState {
    type Action = LoginAction;

    fn reduce(self: std::rc::Rc<Self>, action: Self::Action) -> std::rc::Rc<Self> {
        let mut next = (*self).clone();

        match action {
            LoginAction::SetUsername(value) => next.username = value,
            LoginAction::SetPassword(value) => next.password = value,
            LoginAction::BlurUsername => next.touched_username = true,
            LoginAction::BlurPassword => next.touched_password = true,
            LoginAction::SubmitAttempt => next.submit_attempted = true,
            LoginAction::SubmitStarted => {
                next.submitting = true;
                next.error = None
            }
            LoginAction::SubmitFailed(error) => {
                next.submitting = false;
                next.error = Some(error)
            }
            LoginAction::SubmitDone => next.submitting = false,
        };

        next.into()
    }
}

#[derive(Properties, PartialEq)]
pub struct FieldProps {
    pub name: AttrValue,
    pub label: AttrValue,

    #[prop_or_default]
    pub placeholder: AttrValue,
    #[prop_or_default]
    pub field_type: AttrValue,
    #[prop_or_default]
    pub invalid: bool,
    #[prop_or_default]
    pub on_update: Callback<String>,
}

#[component(Field)]
fn field(props: &FieldProps) -> Html {
    html! {
        <div class="grid gap-2">
            <label for={props.name.clone()} class="text-xs/relaxed font-medium">
                { &props.label }
            </label>

            <Input id={props.name.clone()} invalid={&props.invalid} on_update={&props.on_update} placeholder={&props.placeholder} field_type={&props.field_type} />
        </div>
    }
}

#[component(Login)]
pub fn login() -> Html {
    let state = use_reducer(LoginState::default);
    let navigator = use_navigator().unwrap();

    let on_username = {
        let state = state.clone();
        Callback::from(move |value: String| state.dispatch(LoginAction::SetUsername(value)))
    };

    let on_password = {
        let state = state.clone();
        Callback::from(move |value: String| state.dispatch(LoginAction::SetPassword(value)))
    };

    let on_submit = {
        let state = state.clone();
        let navigator = navigator.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            state.dispatch(LoginAction::SubmitAttempt);

            if !state.can_submit() {
                return;
            }

            state.dispatch(LoginAction::SubmitStarted);
            spawn_local({
                let state = state.clone();
                let navigator = navigator.clone();

                async move {
                    let result =
                        login_mutation(state.username.clone(), state.password.clone()).await;

                    match result {
                        Ok(status) => match status {
                            Success(_) => {
                                state.dispatch(LoginAction::SubmitDone);
                                navigator.replace(&Route::Index);
                            }
                            InvalidCredentials => {
                                state.dispatch(LoginAction::SubmitFailed(
                                    "Invalid username or password".to_string(),
                                ));
                            }
                        },
                        Err(e) => {
                            state.dispatch(LoginAction::SubmitFailed(e.as_report().to_string()));
                        }
                    }
                }
            });
        })
    };

    html! {
        <form onsubmit={on_submit}>
            <div class="flex flex-col max-w-78 mx-auto mt-6 bg-card py-4 pb-0 gap-4">
                <div class="px-4">
                    <p class="font-medium">{ "Login" }</p>
                    <p class="text-xs text-muted-foreground">{ "Enter username below" }</p>
                </div>

                <div class="flex flex-col gap-6 px-4">
                    <div class="space-y-1">
                        <Field invalid={state.show_username_error()} on_update={on_username} name="username" label="Username" placeholder="mrfartshit" />
                        if state.show_username_error() {
                            <p class="text-xs text-red-500">{ state.username_error().unwrap_or_default() }</p>
                        }
                    </div>

                    <div class="space-y-1">
                        <Field invalid={state.show_password_error()} on_update={on_password} name="password" label="Password" placeholder="Password" field_type="password" />
                        if state.show_password_error() {
                            <p class="text-xs text-red-500">{ state.password_error().unwrap_or_default() }</p>
                        }
                    </div>

                    if let Some(error) = &state.error {
                        <p class="text-xs text-red-500">{ error }</p>
                    }
                </div>

                <div class="flex items-center bg-secondary p-4 border-t">
                    if state.submitting {
                        <Button disabled={true} class="w-full py-0.5 flex justify-center"><Loader class="animate-spin" /></Button>
                    } else {
                        <Button button_type={"submit"} class="w-full py-0.5">{ "Login" }</Button>
                    }
                </div>
            </div>
        </form>
    }
}
