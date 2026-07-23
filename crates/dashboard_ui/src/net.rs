use std::{collections::HashMap, rc::Rc};

use gloo::{net::http::Request, timers::callback::Interval};
use types::{
    Backup, BackupRequest, LoginRequest, Report, ServerStatus, StatusResponse, SyncRequest, User,
    World,
};
use web_sys::{RequestCredentials, js_sys::Date};

use errors::{NetError, NetErrorKind};
use yew::{
    Callback, UseStateHandle, hook, platform::spawn_local, use_effect_with, use_mut_ref, use_state,
};

pub enum AuthStatus {
    Authenticated(User),
    Unauthenticated,
}

pub enum LoginStatus {
    Success(User),
    InvalidCredentials,
}

#[derive(Clone, PartialEq)]
pub struct QueryOptions {
    pub stale_time: f64,
    pub refetch_interval: u32,
    pub enabled: bool,
}

impl Default for QueryOptions {
    fn default() -> Self {
        Self {
            stale_time: 10000.0,
            enabled: true,
            refetch_interval: 10000,
        }
    }
}

pub struct QueryState<T, E> {
    pub data: Option<Rc<T>>,
    pub error: Option<Rc<E>>,
    pub fetching: bool,
    pub stale: bool,
    pub last_fetched: Option<f64>,
}

impl<T, E> Clone for QueryState<T, E> {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
            error: self.error.clone(),
            fetching: self.fetching,
            stale: self.stale,
            last_fetched: self.last_fetched,
        }
    }
}

#[hook]
pub fn use_query<T, E, F, Fut>(get: F, options: QueryOptions) -> UseStateHandle<QueryState<T, E>>
where
    T: 'static,
    E: 'static,
    F: Fn() -> Fut + 'static,
    Fut: std::future::Future<Output = Result<T, E>> + 'static,
{
    let state = use_state(|| QueryState {
        data: None,
        error: None,
        fetching: false,
        stale: true,
        last_fetched: None,
    });

    let fetching = use_mut_ref(|| false);
    let get = Rc::new(get);

    let fetch = Callback::from({
        let state = state.clone();
        let fetching = fetching.clone();
        let get = get.clone();

        move |_| {
            if *fetching.borrow() {
                return;
            }
            *fetching.borrow_mut() = true;

            spawn_local({
                let state = state.clone();
                let fetching = fetching.clone();
                let get = get.clone();

                async move {
                    state.set(QueryState {
                        fetching: true,
                        ..(*state).clone()
                    });

                    match get().await {
                        Ok(data) => state.set(QueryState {
                            data: Some(data.into()),
                            error: None,
                            fetching: false,
                            stale: false,
                            last_fetched: Some(Date::now()),
                        }),
                        Err(e) => state.set(QueryState {
                            error: Some(e.into()),
                            fetching: false,
                            ..(*state).clone()
                        }),
                    }

                    *fetching.borrow_mut() = false;
                }
            });
        }
    });

    use_effect_with(options.enabled, {
        let fetch = fetch.clone();
        let state = state.clone();

        move |_| {
            if !options.enabled {
                return;
            }

            let now = Date::now();
            let stale = state
                .last_fetched
                .map(|t| now - t > options.stale_time)
                .unwrap_or(true);

            if stale {
                fetch.emit(())
            }
        }
    });

    use_effect_with(options.enabled, {
        let fetch = fetch.clone();
        move |_| {
            let handle = if options.enabled {
                Some(Interval::new(options.refetch_interval, move || {
                    fetch.emit(())
                }))
            } else {
                None
            };

            move || drop(handle)
        }
    });

    return state;
}

fn error_for_status(status: u16) -> Option<NetError> {
    match status {
        401 => Some(NetError::unauthenticated()),
        403 => Some(NetError::forbidden()),
        404 => Some(NetError::not_found()),
        500..=599 => Some(NetError::internal()),
        _ => None,
    }
}

pub async fn sync_world(
    from_server_name: World,
    destination_server_name: World,
) -> Result<(), NetError> {
    let response = Request::post("/api/sync_world")
        .credentials(RequestCredentials::Include)
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&SyncRequest {
            from_server_name,
            destination_server_name,
        })?)?
        .send()
        .await?;

    match error_for_status(response.status()) {
        Some(err) => Err(err),
        _ => Ok(()),
    }
}

pub async fn backup_world(server_name: World, backup_file_name: String) -> Result<(), NetError> {
    let response = Request::post("/api/backup_world")
        .credentials(RequestCredentials::Include)
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&BackupRequest {
            server_name,
            backup_file_name,
        })?)?
        .send()
        .await?;

    match error_for_status(response.status()) {
        Some(err) => Err(err),
        _ => Ok(()),
    }
}

pub async fn get_operations() -> Result<Vec<Report>, NetError> {
    let response = Request::get("/api/operation_log")
        .credentials(RequestCredentials::Include)
        .send()
        .await?;

    match error_for_status(response.status()) {
        Some(err) => Err(err),
        _ => Ok(response.json().await?),
    }
}

pub async fn get_backups() -> Result<Vec<Backup>, NetError> {
    let response = Request::get("/api/backups")
        .credentials(RequestCredentials::Include)
        .send()
        .await?;

    match error_for_status(response.status()) {
        Some(err) => Err(err),
        _ => Ok(response.json().await?),
    }
}

pub async fn backup_status() -> Result<StatusResponse, NetError> {
    let response = Request::get("/api/status")
        .credentials(RequestCredentials::Include)
        .send()
        .await?;

    match error_for_status(response.status()) {
        Some(err) => Err(err),
        _ => Ok(response.json().await?),
    }
}

pub async fn world_status() -> Result<HashMap<String, ServerStatus>, NetError> {
    let response = Request::get("/api/world_status")
        .credentials(RequestCredentials::Include)
        .send()
        .await?;

    match error_for_status(response.status()) {
        Some(err) => Err(err),
        _ => Ok(response.json().await?),
    }
}

pub async fn login(username: String, auth: String) -> Result<LoginStatus, NetError> {
    let response = Request::post("/api/auth/login")
        .credentials(RequestCredentials::Include)
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&LoginRequest { username, auth })?)?
        .send()
        .await?;

    match error_for_status(response.status()) {
        Some(err) => match err.inner() {
            NetErrorKind::Unauthenticated | NetErrorKind::Forbidden => {
                Ok(LoginStatus::InvalidCredentials)
            }
            _ => Err(err),
        },

        _ => Ok(LoginStatus::Success(response.json().await?)),
    }
}

pub async fn logout() -> Result<(), NetError> {
    let response = Request::post("/api/auth/logout").send().await?;

    match response.status() {
        200 => Ok(()),
        _ => Err(NetError::internal()),
    }
}

pub async fn get_auth_status() -> Result<AuthStatus, NetError> {
    let response = Request::get("/api/auth/me")
        .credentials(RequestCredentials::Include)
        .header("Content-Type", "application/json")
        .send()
        .await?;

    match error_for_status(response.status()) {
        Some(err) => match err.inner() {
            NetErrorKind::Unauthenticated | NetErrorKind::Forbidden => {
                Ok(AuthStatus::Unauthenticated)
            }
            _ => Err(err),
        },

        _ => Ok(AuthStatus::Authenticated(response.json().await?)),
    }
}

pub mod errors {
    use thiserror::Error;
    use thiserror_ext::{Box, Construct};

    #[derive(Error, Construct, Box, Debug)]
    #[thiserror_ext(newtype(name = NetError))]
    pub enum NetErrorKind {
        #[error("network error")]
        Network(#[from] gloo::net::Error),
        #[error("json error")]
        Json(#[from] serde_json::Error),
        #[error("unauthenticated")]
        Unauthenticated,
        #[error("forbidden")]
        Forbidden,
        #[error("not found")]
        NotFound,

        #[error("internal server error")]
        Internal,
    }
}
