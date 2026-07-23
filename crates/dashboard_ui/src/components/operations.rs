use gloo::timers::callback::Timeout;
use thiserror_ext::AsReport;
use tw_merge::tw_merge;
use types::World;
use web_sys::js_sys::Date;
use yew::{
    Callback, Html, Reducible, component, html, platform::spawn_local, use_reducer, use_state,
};

use crate::{
    components::{
        display::button::Button,
        dropdown::{Dropdown, DropdownItem},
        modal::Modal,
    },
    icons::Loader,
    net::{QueryOptions, backup_world, get_backups, sync_world, use_query},
};

#[derive(Clone, PartialEq)]
enum BackupAction {
    Idle,
    Start,
    Success,
    Error(String),
}

#[derive(Clone)]
struct BackupState {
    generation: u32,
    current: BackupAction,
}

impl Reducible for BackupState {
    type Action = BackupAction;

    fn reduce(self: std::rc::Rc<Self>, action: Self::Action) -> std::rc::Rc<Self> {
        let mut next = (*self).clone();

        match action {
            BackupAction::Start => {
                next.generation += 1;
                next.current = BackupAction::Start
            }
            BackupAction::Success => next.current = BackupAction::Success,
            BackupAction::Idle => next.current = BackupAction::Idle,
            BackupAction::Error(e) => next.current = BackupAction::Error(e),
        };

        next.into()
    }
}

fn format_bytes(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;

    let b = bytes as f64;

    if b >= GB {
        format!("{:.1}G", b / GB)
    } else if b >= MB {
        format!("{:.1}MB", b / MB)
    } else if b >= KB {
        format!("{:.1}KB", b / KB)
    } else {
        format!("{bytes}B")
    }
}

fn format_date(iso: &str) -> String {
    const MONTHS: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];

    let date = Date::new(&iso.into());
    let month = MONTHS[date.get_month() as usize];
    let day = date.get_date();

    let hours = date.get_hours();
    let minutes = date.get_minutes();

    let (hour_12, suffix) = match hours {
        0 => (12, "AM"),
        1..=11 => (hours, "AM"),
        12 => (12, "PM"),
        _ => (hours - 12, "PM"),
    };

    format!("{month} {day}, {hour_12}:{minutes:02} {suffix}")
}

#[component(Sync)]
pub fn sync() -> Html {
    let open = use_state(|| false);
    let mode = use_state(|| "regular");

    let state = use_reducer(|| BackupState {
        generation: 0,
        current: BackupAction::Idle,
    });

    let options = vec![
        DropdownItem {
            id: "regular",
            content: html! { { "Quick Sync"} },
        },
        DropdownItem {
            id: "save",
            content: html! { {"Save & Sync"} },
        },
    ];

    let update_selected = Callback::from({
        let open = open.clone();
        let mode = mode.clone();

        move |id: &'static str| {
            match id {
                "regular" => mode.set("regular"),
                "save" => mode.set("save"),
                _ => unreachable!(),
            };
            open.set(true);
        }
    });

    let on_cancel = Callback::from({
        let open = open.clone();
        move |_| {
            open.set(false);
        }
    });

    let on_confirm = Callback::from({
        let open = open.clone();
        let state = state.clone();
        let mode = mode.clone();

        move |_| {
            open.set(false);

            spawn_local({
                let state = state.clone();
                let mode = mode.clone();

                async move {
                    state.dispatch(BackupAction::Start);
                    let generation = state.generation;

                    let result = if *mode == "regular" {
                        backup_world(World::Creative, "latest.tar.gz".to_string()).await
                    } else {
                        sync_world(World::Main, World::Creative).await
                    };

                    match result {
                        Ok(_) => {
                            state.dispatch(BackupAction::Success);
                            Timeout::new(4000, move || {
                                if generation != state.generation {
                                    return;
                                }

                                state.dispatch(BackupAction::Idle);
                            })
                            .forget();
                        }
                        Err(e) => state.dispatch(BackupAction::Error(e.as_report().to_string())),
                    }
                }
            });
        }
    });

    let button_class = tw_merge!(
        "px-2 py-0.5 rounded-md",
        match state.current {
            BackupAction::Success => "bg-green-500/40",
            BackupAction::Error(_) => "bg-red-500/40 hover:bg-red-500/40",
            _ => "",
        }
    );

    let disabled = !matches!(state.current, BackupAction::Idle | BackupAction::Error(_));

    html! {
        <>
        <Modal
            title="World sync"
            open={*open}
            message="Sync the creative world?"
            {on_cancel}
            {on_confirm}
        />

        <Dropdown<&'static str> {disabled} {options} {update_selected} item_class="text-xs text-nowrap">
            <Button {disabled} class={button_class}>
                <p>{ "Sync" }</p>
            </Button>
        </Dropdown<&'static str>>
        </>
    }
}

#[component(Backup)]
pub fn backup() -> Html {
    let backups_query = use_query(
        get_backups,
        QueryOptions {
            enabled: true,
            refetch_interval: 20000,
            stale_time: 20000.0,
        },
    );
    let state = use_reducer(|| BackupState {
        generation: 0,
        current: BackupAction::Idle,
    });

    let selected_backup = use_state::<Option<types::Backup>, _>(|| None);
    let open = use_state(|| false);

    fn error_state() -> Html {
        html! { <Button disabled={true} class="px-2 py-0.5 rounded-md bg-destructive">{ "Error getting backups" }</Button> }
    }

    fn loading_state() -> Html {
        html! { <Button disabled={true} class="px-2 py-0.5 rounded-md bg-muted-foreground"><Loader class="animate-spin" /></Button> }
    }

    if backups_query.error.is_some() {
        return error_state();
    }

    let Some(data) = backups_query.data.as_ref() else {
        return loading_state();
    };

    let options: Vec<DropdownItem<String>> = data
        .iter()
        .map(|b| DropdownItem {
            id: b.filename.clone(),
            content: html! { { format!("{} - {}", format_date(&b.date), format_bytes(b.bytes)) } },
        })
        .collect();

    let update_selected = Callback::from({
        let selected_backup = selected_backup.clone();
        let open = open.clone();
        let data = data.clone();

        move |id: String| {
            let selection = data.iter().find(|b| b.filename == id).cloned();
            selected_backup.set(selection);
            open.set(true)
        }
    });

    let on_cancel = Callback::from({
        let open = open.clone();
        move |_| open.set(false)
    });

    let on_confirm = Callback::from({
        let open = open.clone();
        let selected_backup = selected_backup.clone();
        let state = state.clone();

        move |_| {
            open.set(false);

            spawn_local({
                let selected_backup = selected_backup.clone();
                let state = state.clone();
                async move {
                    state.dispatch(BackupAction::Start);
                    let generation = state.generation;

                    let result = backup_world(
                        World::Main,
                        selected_backup.as_ref().unwrap().filename.clone(),
                    )
                    .await;

                    match result {
                        Ok(_) => {
                            state.dispatch(BackupAction::Success);
                            Timeout::new(4000, move || {
                                if state.generation != generation {
                                    return;
                                }

                                state.dispatch(BackupAction::Idle)
                            })
                            .forget();
                        }
                        Err(e) => state.dispatch(BackupAction::Error(e.as_report().to_string())),
                    }
                }
            });
        }
    });

    let on_closed = Callback::from({
        let selected_backup = selected_backup.clone();
        move |_| selected_backup.set(None)
    });

    let button_class = tw_merge!(
        "px-2 py-0.5 rounded-md",
        match state.current {
            BackupAction::Success => "bg-green-500/40",
            BackupAction::Error(_) => "bg-red-500/40 hover:bg-red-500/40",
            _ => "",
        }
    );

    let disabled = !matches!(state.current, BackupAction::Idle | BackupAction::Error(_));

    html! {
        <>
        <Modal
            title="World Backup"
            open={*open}
            message={selected_backup.as_ref().map(|b| format!("Backup main world at {}? ({})", format_date(&b.date), format_bytes(b.bytes))).unwrap_or("Select a backup".to_string())}
            {on_cancel}
            {on_confirm}
            {on_closed}
        />

        <Dropdown<String> {disabled} {options} {update_selected} item_class="text-xs text-nowrap">
            <Button {disabled} class={button_class}>
                <p>{ match state.current {
                    BackupAction::Start => "Backing up...",
                    BackupAction::Success => "Backed up",
                    BackupAction::Error(_) => "Error sending backing request",
                    _ => "Backup"
                }}</p>
            </Button>
        </Dropdown<String>>
        </>
    }
}
