use gloo::timers::callback::Timeout;
use thiserror_ext::AsReport;
use types::{Server, ServerStatus, World};
use web_sys::js_sys::Date;
use yew::{
    Callback, Html, Properties, Reducible, component, html, platform::spawn_local, use_reducer,
    use_state,
};

use crate::{
    components::{
        display::{button::Button, input::Input},
        dropdown::{Dropdown, DropdownItem},
        modal::Modal,
    },
    icons::Loader,
    net::{QueryOptions, backup_status, backup_world, get_backups, use_query, world_status},
};

pub mod login;

#[derive(Properties, PartialEq)]
struct IndicatorProps {
    status: ServerStatus,
}

#[derive(Clone)]
enum BackupAction {
    Idle,
    StartBackup,
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
            BackupAction::StartBackup => next.generation += 1,
            BackupAction::Success => next.current = BackupAction::Success,
            BackupAction::Idle => next.current = BackupAction::Idle,
            BackupAction::Error(e) => next.current = BackupAction::Error(e),
        };

        next.into()
    }
}

fn player_head(uuid: &str) -> String {
    format!("https://mc-heads.net/avatar/{uuid}/100")
}

#[derive(Properties, PartialEq)]
struct BackupProps {
    #[prop_or_default]
    active: bool,
}

#[component(StatusIndicator)]
fn status_indicator(props: &BackupProps) -> Html {
    if props.active {
        html! {
            <span class="h-2 w-2 bg-amber-500 animate-pulse rounded-full"></span>
        }
    } else {
        html! {}
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

#[component(Operations)]
fn operations() -> Html {
    let query = use_query(world_status, QueryOptions::default());
    let statuses = query.data.as_ref();

    #[component(Indicator)]
    fn indicator(IndicatorProps { status }: &IndicatorProps) -> Html {
        match status {
            ServerStatus::Running => html! {
                <span class="text-[11px] rounded-full px-2 py-0.5 bg-green-500/15 text-green-400">{ "Running" }</span>
            },
            ServerStatus::Stopped => html! {
                <span class="text-[11px] rounded-full px-2 py-0.5 bg-gray-500/10 text-gray-500">{ "Stopped" }</span>
            },
            ServerStatus::Starting => html! {
                <span class="text-[11px] rounded-full px-2 py-0.5 bg-yellow-500 text-yellow-400 animate-pulse">{ "Starting" }</span>
            },
            ServerStatus::Unknown => html! {
                <span class="text-[11px] rounded-full px-2 py-0.5 bg-gray-500/10 text-gray-500">{ "Fetching..." }</span>
            },
        }
    }

    #[component(Backup)]
    fn backup() -> Html {
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
                        let generation = state.generation;
                        state.dispatch(BackupAction::StartBackup);

                        let result = backup_world(
                            World::Main,
                            selected_backup.as_ref().unwrap().filename.clone(),
                        )
                        .await;

                        match result {
                            Ok(_) => {
                                state.dispatch(BackupAction::Success);
                                Timeout::new(1000, move || {
                                    if state.generation != generation {
                                        return;
                                    }

                                    state.dispatch(BackupAction::Idle)
                                })
                                .forget();
                            }
                            Err(e) => {
                                state.dispatch(BackupAction::Error(e.as_report().to_string()))
                            }
                        }
                    }
                });
            }
        });

        let on_closed = Callback::from({
            let selected_backup = selected_backup.clone();
            move |_| selected_backup.set(None)
        });

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

            <Dropdown<String> {options} {update_selected} item_class="text-xs text-nowrap">
                <Button class="px-2 py-0.5 rounded-md">{ "Backup" }</Button>
            </Dropdown<String>>
            </>
        }
    }

    html! {
        <section class="rounded-lg border bg-background p-4">
            <div class="flex items-center mb-3">
                <h2 class="text-sm font-semibold uppercase tracking-wide text-muted-foreground">{ "Operations" }</h2>
            </div>

            <div class="space-y-3">
                <div class="rounded-md border bg-card px-3 py-2 flex items-center justify-between gap-3">
                    <div class="flex items-center gap-2 min-w-0">
                        <span class="text-sm font-medium">{ "main" }</span>
                        <Indicator status={statuses.map(|s| (*s.get("main").unwrap()).clone()).unwrap_or(ServerStatus::Unknown)}/>
                    </div>

                    <Backup />
                </div>

                <div class="rounded-md border bg-card px-3 py-2 flex items-center justify-between gap-3">
                    <div class="flex items-center gap-2 min-w-0">
                        <span class="text-sm font-medium">{ "creative" }</span>
                        <Indicator status={statuses.map(|s| (*s.get("creative").unwrap()).clone()).unwrap_or(ServerStatus::Unknown)}/>
                    </div>

                    <Button class="px-2 py-0.5 rounded-md">{ "Sync" }</Button>
                </div>
            </div>
        </section>
    }
}

#[component(State)]
fn state() -> Html {
    let status = use_query(
        backup_status,
        QueryOptions {
            enabled: true,
            refetch_interval: 20000,
            stale_time: 20000.0,
        },
    );
    let backing_up = status.data.as_ref().map(|s| s.backing_up).unwrap_or(false);

    html! {
        <section class="rounded-lg border bg-background p-4">
            <div class="flex items-center justify-between mb-3">
                <h2 class="text-sm font-semibold uppercase tracking-wide text-muted-foreground">{ "State" }</h2>
                <span class="inline-flex items-center gap-1 text-xs text-muted-foreground">
                    <StatusIndicator active={backing_up}/>
                    if backing_up {
                        { "Running..." }
                    }
                </span>
            </div>

            <div class="grid grid-cols-2 gap-2">
                <div class="rounded-md border bg-card px-3 py-2">
                    <p class="text-[11px] text-muted-foreground">{ "Main world backup" }</p>
                    <p class="text-sm font-semibold text-muted-foreground">{ "unfinished" }</p>
                </div>
                <div class="rounded-md border bg-card px-3 py-2">
                    <p class="text-[11px] text-muted-foreground">{ "Creative world sync" }</p>
                    <p class="text-sm font-semibold text-muted-foreground">{ "unfinished" }</p>
                </div>
            </div>
        </section>

    }
}

#[component(Whitelist)]
fn whitelist() -> Html {
    let whitelist_players = [
        ("069a79f4-44e9-4726-a5be-fca90e38aaf5", "Notch", "main"),
        (
            "853c80ef-3c37-49fd-aa49-938b674adae6",
            "mrfartshit",
            "creative",
        ),
    ];

    let options = vec![
        DropdownItem {
            id: Server::Main,
            content: html! { { "main" } },
        },
        DropdownItem {
            id: Server::Creative,
            content: html! { { "creative" } },
        },
    ];

    let selected_world = use_state(|| "main");

    let update_selected = Callback::from({
        let selected_world = selected_world.clone();

        move |id: Server| match id {
            Server::Main => selected_world.set("main"),
            Server::Creative => selected_world.set("creative"),
        }
    });

    html! {
        <section class="lg:col-span-2 rounded-lg border bg-background p-4">
            <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-3 mb-3">
                <h2 class="text-sm font-semibold uppercase tracking-wide text-muted-foreground">{ "Whitelist" }</h2>

                <div class="flex items-center gap-2">
                    <Dropdown<Server> {options} {update_selected}>
                        <span class="text-xs rounded-md border bg-card px-2 py-1.5">{ *selected_world }</span>
                    </Dropdown<Server>>
                    <Input
                        field_type="text"
                        placeholder="search player"
                        class="text-xs rounded-md border bg-card px-2 py-1.5 w-36"
                    />
                </div>
            </div>

            <div class="max-h-72 overflow-y-auto space-y-2 pr-1">
                { for whitelist_players.iter().map(|(uuid, username, world)| html! {
                    <div class="rounded-md border bg-card px-3 py-2 flex items-center justify-between gap-3">
                        <div class="flex items-center gap-3 min-w-0">
                            <img src={player_head(uuid)} alt={format!("{} avatar", username)} class="h-9 w-9 rounded-sm border bg-muted object-cover" />
                            <div class="min-w-0">
                                <p class="text-sm font-medium truncate">{ *username }</p>
                                <p class="text-[11px] text-muted-foreground">{ format!("world: {}", world) }</p>
                            </div>
                        </div>

                        <Button class="px-2 py-1 rounded-md bg-destructive text-destructive-foreground hover:bg-destructive/90">{ "Remove" }</Button>
                    </div>
                }) }
            </div>

            <div class="mt-3 pt-3 border-t flex flex-col sm:flex-row gap-2">
                <Input placeholder="add username" />
                <Button class="px-3 py-1.5 rounded-md sm:w-auto w-full">{ "Add" }</Button>
            </div>
        </section>
    }
}

#[component(Index)]
pub fn index() -> Html {
    let online_players = [
        ("Notch", "creative"),
        ("mrfartshit", "main"),
        ("thedarkknight15963", "creative"),
    ];

    html! {
        <div class="w-full max-w-6xl mx-auto px-4 pb-10">
            <div class="mt-6 rounded-xl border bg-card shadow-sm overflow-hidden">
                <div class="px-6 py-5 border-b bg-muted/20">
                    <h1 class="text-xl font-semibold tracking-tight">{ "Management" }</h1>
                    <p class="text-xs/relaxed font-medium text-muted-foreground mt-1">{ "Minecraft Server Console" }</p>
                </div>

                <div class="p-6 grid grid-cols-1 lg:grid-cols-2 gap-5 items-start">
                    <Operations />
                    <State />
               </div>

                <div class="px-6 pb-6 grid grid-cols-1 lg:grid-cols-3 gap-5">
                    <Whitelist />

                    <section class="rounded-lg border bg-background p-4">
                        <div class="flex items-center justify-between mb-3">
                            <h2 class="text-sm font-semibold uppercase tracking-wide text-muted-foreground">{ "Online" }</h2>
                            <span class="text-xs px-2 py-0.5 rounded-full bg-emerald-500/15 text-emerald-700">{ "2 Players" }</span>
                        </div>

                        <div class="space-y-2">
                            { for online_players.iter().map(|(username, world)| html! {
                                <div class="rounded-md border bg-card px-3 py-2">
                                    <div class="flex items-center justify-between gap-2">
                                        <p class="text-sm font-medium truncate">{ *username }</p>
                                        // <p class="text-[11px] text-muted-foreground">{ format!("{} online", duration) }</p>
                                    </div>
                                    <p class="text-[11px] text-muted-foreground mt-1">{ format!("World: {}", world) }</p>
                                </div>
                            }) }
                        </div>

                        <div class="mt-3 pt-3 border-t">
                            <Button class="w-full px-2 py-1.5 rounded-md bg-secondary text-secondary-foreground hover:bg-secondary/80">{ "View Full Player List" }</Button>
                        </div>
                    </section>
                </div>
            </div>
        </div>
    }
}
