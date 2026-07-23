use chrono::{DateTime, Utc};
use types::{Operation, Server, ServerStatus};
use yew::{Callback, Html, Properties, component, html, use_state};

use crate::{
    components::{
        display::{button::Button, input::Input},
        dropdown::{Dropdown, DropdownItem},
        operations::{Backup, Sync},
    },
    net::{QueryOptions, backup_status, get_operations, use_query, world_status},
};

pub mod login;

#[derive(Properties, PartialEq)]
struct IndicatorProps {
    status: ServerStatus,
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

fn format_time(timestamp: DateTime<Utc>) -> String {
    let now = Utc::now();
    let seconds = (now - timestamp).num_seconds();

    if seconds <= 0 {
        return "just now".to_string();
    }

    let minutes = seconds / 60;
    let hours = minutes / 60;
    let days = hours / 24;

    if minutes < 1 {
        "just now".to_string()
    } else if minutes < 60 {
        format!("{minutes}m ago")
    } else if hours < 24 {
        let remaining_minutes = minutes % 60;

        if remaining_minutes == 0 {
            format!("{hours}hr ago")
        } else {
            format!("{hours}hr {remaining_minutes}m ago")
        }
    } else {
        let remaining_hours = hours % 24;

        if remaining_hours == 0 {
            format!("{days}d ago")
        } else {
            format!("{days}d {remaining_hours}hr ago")
        }
    }
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
                <span class="text-[11px] rounded-full px-2 py-0.5 bg-yellow-500/15 text-yellow-400 animate-pulse">{ "Starting" }</span>
            },
            ServerStatus::Unknown => html! {
                <span class="text-[11px] rounded-full px-2 py-0.5 bg-gray-500/10 text-gray-500">{ "Fetching..." }</span>
            },
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

                    <Sync />
                </div>
            </div>
        </section>
    }
}

#[component(StateSkeleton)]
fn state_skeleton() -> Html {
    html! {
        <div class="grid grid-cols-2 gap-2 animate-pulse">
            <div class="rounded-md border bg-card px-3 py-2 space-y-2">
                <p class="text-[11px] text-muted-foreground">{ "Main world backup" }</p>
                <div class="h-4 w-20 rounded bg-muted"></div>
            </div>
            <div class="rounded-md border bg-card px-3 py-2 space-y-2">
                <p class="text-[11px] text-muted-foreground">{ "Creative world sync" }</p>
                <div class="h-4 w-20 rounded bg-muted"></div>
            </div>
        </div>
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

    #[component(OperationLog)]
    fn operation_log() -> Html {
        let operations = use_query(
            get_operations,
            QueryOptions {
                enabled: true,
                refetch_interval: 60000,
                stale_time: 60000.0,
            },
        );

        let Some(data) = operations.data.as_ref() else {
            return html! {
                <StateSkeleton />
            };
        };

        html! {
            <div class="grid grid-cols-2 gap-2">
                <div class="rounded-md border bg-card px-3 py-2">
                    <p class="text-[11px] text-muted-foreground">{ "Main world backup" }</p>
                    <p class="text-sm font-semibold">
                        { data.iter().find(|r| r.operation == Operation::Backup).map(|r| format_time(r.timestamp)) }
                    </p>
                </div>

                <div class="rounded-md border bg-card px-3 py-2">
                    <p class="text-[11px] text-muted-foreground">{ "Creative world sync" }</p>
                    <p class="text-sm font-semibold">
                        { data.iter().find(|r| r.operation == Operation::Sync).map(|r| format_time(r.timestamp)) }
                    </p>
                </div>
            </div>
        }
    }

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

            <OperationLog />
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
