use std::rc::Rc;

use tw_merge::tw_merge;
use yew::{
    AttrValue, Callback, Html, MouseEvent, Properties, Reducible, component, html, use_effect_with,
    use_reducer,
};

use super::display::button::Button;

#[derive(Clone, PartialEq, Debug)]
enum ModalState {
    Closed,
    Opening,
    Open,
    Closing,
}

enum Action {
    SetOpen(bool),
    AnimationEnd,
}

impl Reducible for ModalState {
    type Action = Action;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let next = match (self.as_ref(), action) {
            (ModalState::Closed, Action::SetOpen(true)) => ModalState::Opening,
            (ModalState::Closing, Action::SetOpen(true)) => ModalState::Opening,

            (ModalState::Open, Action::SetOpen(false)) => ModalState::Closing,
            (ModalState::Opening, Action::SetOpen(false)) => ModalState::Closing,

            (ModalState::Opening, Action::AnimationEnd) => ModalState::Open,
            (ModalState::Closing, Action::AnimationEnd) => ModalState::Closed,

            _ => (*self).clone(),
        };

        next.into()
    }
}

#[derive(Properties, PartialEq)]
pub struct ModalProps {
    pub title: AttrValue,
    #[prop_or_default]
    pub message: AttrValue,
    #[prop_or_default]
    pub open: bool,
    #[prop_or_default]
    pub on_cancel: Callback<()>,
    #[prop_or_default]
    pub on_confirm: Callback<()>,
    #[prop_or_default]
    pub on_closed: Callback<()>,
}

fn prop_emit_unit(prop: Callback<()>) -> Callback<MouseEvent> {
    Callback::from(move |_| prop.emit(()))
}

#[component(Modal)]
pub fn modal(props: &ModalProps) -> Html {
    let cancel = prop_emit_unit(props.on_cancel.clone());
    let confirm = prop_emit_unit(props.on_confirm.clone());

    let state = use_reducer(|| {
        if props.open {
            ModalState::Open
        } else {
            ModalState::Closed
        }
    });

    use_effect_with(props.open, {
        let state = state.clone();
        move |open| {
            state.dispatch(Action::SetOpen(*open));
        }
    });

    let on_animation_end = Callback::from({
        let state = state.clone();
        let on_closed = props.on_closed.clone();

        move |_| {
            let closing = *state == ModalState::Closing;
            state.dispatch(Action::AnimationEnd);

            if closing {
                on_closed.emit(())
            }
        }
    });

    if *state == ModalState::Closed {
        return html! {};
    }

    let root_class = tw_merge!(
        "fixed inset-0 z-50 flex items-center justify-center p-4",
        if *state == ModalState::Opening || *state == ModalState::Open {
            "pointer-events-auto"
        } else {
            "pointer-events-none"
        }
    );

    let modal_class = tw_merge!(
        "relative z-10 w-full max-w-md border bg-background shadow-xl p-4",
        match *state {
            ModalState::Opening => "animate-[modal-in_150ms_ease-out_forwards]",
            ModalState::Open => "opacity-100 translate-y-0 scale-100",
            ModalState::Closing => "animate-[modal-out_150ms_ease-in_forwards]",
            ModalState::Closed => "opacity-0 translate-y-1 scale-95",
        }
    );

    let backdrop_class = tw_merge!(
        "absolute inset-0 bg-black/55 backdrop-blur-[1px] cursor-default",
        match *state {
            ModalState::Opening => "animate-[backdrop-in_150ms_ease-out_forwards]",
            ModalState::Open => "opacity-100 backdrop-blur-[1px]",
            ModalState::Closing => "animate-[backdrop-out_150ms_ease-in_forwards]",
            ModalState::Closed => "opacity-0 backdrop-blur-none",
        }
    );

    html! {
        <div
            class={root_class}
            aria-modal={true}
            role="dialog"
        >
            <button type="button" class={backdrop_class} onclick={cancel.clone()} />
            <div class={modal_class} onanimationend={on_animation_end}>
                <h3 class="text-base font-semibold">{ &props.title }</h3>
                <p class="mt-2 text-sm text-muted-foreground">{ &props.message }</p>

                <div class="mt-4 flex justify-end gap-2">
                    <Button onclick={cancel} class="p-0.5 rounded-sm">{ "Cancel" }</Button>
                    <Button onclick={confirm} class="p-0.5 rounded-sm">{ "Confirm" }</Button>
                </div>
            </div>
        </div>
    }
}
