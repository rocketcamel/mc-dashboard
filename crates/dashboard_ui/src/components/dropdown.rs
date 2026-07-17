use gloo::events::EventListener;
use tw_merge::tw_merge;
use web_sys::{Node, wasm_bindgen::JsCast, window};
use yew::{
    AttrValue, Callback, Children, Html, NodeRef, Properties, UseStateHandle, classes, component,
    html, use_effect_with, use_node_ref, use_state,
};

#[derive(Clone, PartialEq)]
pub struct DropdownItem<T> {
    pub id: T,
    pub content: Html,
}

#[derive(Properties, PartialEq, Default)]
pub struct DropdownProps<T: PartialEq> {
    #[prop_or_default]
    pub children: Children,
    #[prop_or_default]
    pub label: Option<AttrValue>,
    #[prop_or_default]
    pub item_class: AttrValue,

    #[prop_or_default]
    pub options: Vec<DropdownItem<T>>,
    #[prop_or_default]
    pub update_selected: Callback<T>,
}

fn outside_click(
    open: UseStateHandle<bool>,
    trigger_ref: NodeRef,
    menu_ref: NodeRef,
) -> impl FnOnce() {
    fn clicked(node: &Option<Node>, target: &Option<Node>) -> bool {
        match (node, target) {
            (Some(root), Some(t)) => root.contains(Some(t)),
            _ => false,
        }
    }

    let listener = window().and_then(|w| w.document()).map(|document| {
        EventListener::new(&document, "mousedown", move |event| {
            let target = event.target().and_then(|t| t.dyn_into::<Node>().ok());

            let trigger_node = trigger_ref.cast::<Node>();
            let menu_node = menu_ref.cast::<Node>();

            let inside_trigger = clicked(&trigger_node, &target);
            let inside_menu = clicked(&menu_node, &target);

            if !inside_trigger && !inside_menu {
                open.set(false);
            }
        })
    });

    move || drop(listener)
}

#[component(Dropdown)]
pub fn dropdown<T: Clone + PartialEq + 'static>(props: &DropdownProps<T>) -> Html {
    let open = use_state(|| false);
    let trigger_ref = use_node_ref();
    let menu_ref = use_node_ref();
    let state = if *open { "open" } else { "closed " };

    let toggle = {
        let open = open.clone();
        Callback::from(move |_| open.set(!*open))
    };

    use_effect_with((), {
        let open = open.clone();
        let trigger_ref = trigger_ref.clone();
        let menu_ref = menu_ref.clone();

        move |_| outside_click(open, trigger_ref, menu_ref)
    });

    let state_class = if *open {
        classes!(
            "opacity-100",
            "translate-y-0",
            "scale-100",
            "pointer-events-auto"
        )
    } else {
        classes!(
            "opacity-0",
            "-translate-y-1",
            "scale-95",
            "pointer-events-none"
        )
    };

    let menu_class = classes!(
        "absolute",
        "overflow-y-auto",
        "max-h-72",
        "bg-primary-foreground",
        "mt-1",
        "min-w-32",
        "border",
        "p-1",
        "shadow-lg",
        "z-50",
        "transition",
        "duration-100",
        "ease-out",
        state_class
    );

    html! {
        <div class="relative inline-block" data-state={state}>
            <button type="button"
                onclick={toggle}
                ref={trigger_ref}
                aria-haspopup="menu"
                aria-expanded={open.to_string()}
                class="m-0 p-1 flex h-full w-full"
            >
                { for props.children.iter() }
            </button>

            <div class={menu_class} role="menu" ref={menu_ref}>
                if let Some(label) = &props.label {
                    <p class="border-b flex items-center text-xs px-2 py-1.5 text-muted-foreground">{ label }</p>
                }

               { for props.options.iter().map(|item| {
                   let update_selected = props.update_selected.clone();

                   let onclick = Callback::from({
                        let id = item.id.clone();
                        let open = open.clone();

                         move |_| {
                            update_selected.emit(id.clone());
                            open.set(false);
                        }}
                    );

                   html! {
                       <button type="button" {onclick} class={tw_merge!("flex w-full rounded-md px-2.5 py-1.5 text-left text-sm
                                                              hover:bg-zinc-200 dark:hover:bg-accent mt-1", props.item_class.to_string())}
                        >
                        { item.content.clone() }
                       </button>
                   }
               })}
            </div>
        </div>
    }
}
