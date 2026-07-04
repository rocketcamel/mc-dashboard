use yew::{Html, component, html};

use crate::{components::display::button::Button, styles::merge_styles};

pub mod login;

fn player_head(uuid: &str) -> String {
    format!("https://mc-heads.net/avatar/{uuid}/100")
}

#[component(StatusIndicator)]
fn status_indicator() -> Html {
    html! {
        <span class="h-2 w-2 bg-amber-500 animate-pulse rounded-full"></span>
    }
}

#[component(Operations)]
fn operations() -> Html {
    let world_statuses = [
        ("creative", "running", "bg-emerald-500/15 text-emerald-700"),
        ("main", "running", "bg-emerald-500/15 text-emerald-700"),
    ];

    html! {
        <section class="rounded-lg border bg-background p-4">
            <div class="flex items-center mb-3">
                <h2 class="text-sm font-semibold uppercase tracking-wide text-muted-foreground">{ "Operations" }</h2>
            </div>

            <div class="space-y-3">
                { for world_statuses.iter().map(|(world, status, classes)| html! {
                    <div class="rounded-md border bg-card px-3 py-2 flex items-center justify-between gap-3">
                        <div class="flex items-center gap-2 min-w-0">
                            <span class="text-sm font-medium">{ *world }</span>
                            <span class={format!("text-[11px] rounded-full px-2 py-0.5 {}", *classes)}>{ *status }</span>
                        </div>

                        if *world == "Creative" {
                            <Button class="px-2 py-0.5 rounded-md">{ "Sync" }</Button>
                        } else {
                            <Button class="px-2 py-0.5 rounded-md">{ "Backup" }</Button>
                        }
                    </div>
                }) }
            </div>
        </section>
    }
}

#[component(State)]
fn state() -> Html {
    let operation_feed = [
        ("Backup main world", "2m ago"),
        ("Creative quick sync", "11m ago"),
        ("Whitelist update", "19m ago"),
    ];

    html! {
        <section class="rounded-lg border bg-background p-4">
            <div class="flex items-center justify-between mb-3">
                <h2 class="text-sm font-semibold uppercase tracking-wide text-muted-foreground">{ "State" }</h2>
                <span class="inline-flex items-center gap-1 text-xs text-muted-foreground">
                    <StatusIndicator />
                    { "Running..." }
                </span>
            </div>

            <div class="grid grid-cols-2 gap-2 mb-3">
                <div class="rounded-md border bg-card px-3 py-2">
                    <p class="text-[11px] text-muted-foreground">{ "Last Sync" }</p>
                    <p class="text-sm font-semibold">{ "11m" }</p>
                </div>
                <div class="rounded-md border bg-card px-3 py-2">
                    <p class="text-[11px] text-muted-foreground">{ "Uptime" }</p>
                    <p class="text-sm font-semibold">{ "03:42:10" }</p>
                </div>
            </div>

            <div class="rounded-md border bg-card p-3 space-y-2">
                { for operation_feed.iter().map(|(label, when)| html! {
                    <div class="flex items-center justify-between text-xs">
                        <p class="font-medium">{ *label }</p>
                        <p class="text-muted-foreground">{ *when }</p>
                    </div>
                }) }
            </div>
        </section>

    }
}

#[component(Index)]
pub fn index() -> Html {
    let whitelist_players = [
        ("069a79f4-44e9-4726-a5be-fca90e38aaf5", "Notch", "main"),
        (
            "853c80ef-3c37-49fd-aa49-938b674adae6",
            "mrfartshit",
            "creative",
        ),
    ];

    let online_players = [
        ("BuilderBee", "creative", "18m"),
        ("RedstoneRex", "main", "42m"),
        ("SkyCart", "creative", "7m"),
    ];

    let mut default_style = "bg-black font-medium text-white";
    let mut merge_style = "bg-amber-500 font-regular";

    merge_styles(default_style.as_bytes(), merge_style.as_bytes());

    html! {
        <div class="w-full max-w-6xl mx-auto px-4 pb-10">
            <div class="mt-6 rounded-xl border bg-card shadow-sm overflow-hidden">
                <div class="px-6 py-5 border-b bg-muted/20">
                    <h1 class="text-xl font-semibold tracking-tight">{ "Management" }</h1>
                    <p class="text-xs/relaxed font-medium text-muted-foreground mt-1">{ "Minecraft Server Console" }</p>
                </div>

                <div class="p-6 grid grid-cols-1 lg:grid-cols-2 gap-5">
                    <Operations />
                    <State />
               </div>

                <div class="px-6 pb-6 grid grid-cols-1 lg:grid-cols-3 gap-5">
                    <section class="lg:col-span-2 rounded-lg border bg-background p-4">
                        <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-3 mb-3">
                            <h2 class="text-sm font-semibold uppercase tracking-wide text-muted-foreground">{ "Whitelist Manager" }</h2>

                            <div class="flex items-center gap-2">
                                <select class="text-xs rounded-md border bg-card px-2 py-1.5">
                                    <option value="main">{ "Main" }</option>
                                    <option value="creative">{ "Creative" }</option>
                                </select>
                                <input
                                    type="text"
                                    placeholder="Search player"
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
                                            <p class="text-[11px] text-muted-foreground">{ format!("World: {}", world) }</p>
                                        </div>
                                    </div>

                                    <Button class="px-2 py-1 rounded-md bg-destructive text-destructive-foreground hover:bg-destructive/90">{ "Remove" }</Button>
                                </div>
                            }) }
                        </div>

                        <div class="mt-3 pt-3 border-t flex flex-col sm:flex-row gap-2">
                            <input type="text" placeholder="Add username" class="text-xs rounded-md border bg-card px-2 py-1.5 flex-1" />
                            <Button class="px-3 py-1.5 rounded-md sm:w-auto w-full">{ "Add To Whitelist" }</Button>
                        </div>
                    </section>

                    <section class="rounded-lg border bg-background p-4">
                        <div class="flex items-center justify-between mb-3">
                            <h2 class="text-sm font-semibold uppercase tracking-wide text-muted-foreground">{ "Online Now" }</h2>
                            <span class="text-xs px-2 py-0.5 rounded-full bg-emerald-500/15 text-emerald-700">{ "3 Players" }</span>
                        </div>

                        <div class="space-y-2">
                            { for online_players.iter().map(|(username, world, duration)| html! {
                                <div class="rounded-md border bg-card px-3 py-2">
                                    <div class="flex items-center justify-between gap-2">
                                        <p class="text-sm font-medium truncate">{ *username }</p>
                                        <p class="text-[11px] text-muted-foreground">{ format!("{} online", duration) }</p>
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
