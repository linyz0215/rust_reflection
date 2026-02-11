use std::{ops::Deref, sync::Arc};

use anyhow::Result;
use axum::{Json, Router, extract::{Query, State}, http::StatusCode, response::IntoResponse, routing::get};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use tracing::{info, instrument, level_filters::LevelFilter};
use tracing_subscriber::{Layer as _, fmt::{Layer, format::FmtSpan}, layer::SubscriberExt, util::SubscriberInitExt};


#[derive(Debug, Clone)]
struct AppState {
    data: Arc<DashMap<i32,User>>,
}

#[derive(Deserialize, Debug)]
struct UserFilter {
    #[serde(default = "default_limit")]
    limit: i32,
    #[serde(default="teenager_default")]
    teenager: bool,
    order: Option<String>,
}
pub fn teenager_default() -> bool {
    false
}
pub fn default_limit() -> i32 {
    i32::MAX
}
#[derive(Debug, Serialize, Clone)]
struct User {
    id: i32,
    name: String,
    email: String,
    gender: String,
    teenager: bool,
}
#[tokio::main]
async fn main() -> Result<()> {
    let file_appender = tracing_appender::rolling::daily("tmp", "http_get.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    let file = Layer::new()
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .pretty()
        .with_writer(non_blocking)
        .with_filter(LevelFilter::INFO);

    let console = Layer::new()
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .pretty()
        .with_filter(LevelFilter::DEBUG);

    tracing_subscriber::registry().with(console).with(file).init();

    let state = AppState::new();
    state.insert(1 as i32,User {
        id: 1,
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
        gender: "female".to_string(),
        teenager: false,
    });
    state.insert(2 as i32,User {
        id: 2,
        name: "Bob".to_string(),
        email: "bob@example.com".to_string(),
        gender: "male".to_string(),
        teenager: true,
    });
    state.insert(7 as i32,User {
        id: 7,
        name: "Charlie".to_string(),
        email: "charlie@example.com".to_string(),
        gender: "female".to_string(),
        teenager: false,

    });
    state.insert(5 as i32,User {
        id: 5,
        name: "David".to_string(),
        email: "david@example.com".to_string(),
        gender: "male".to_string(),
        teenager: true,
    });
    let addr = "0.0.0.0:8080";
    let app = Router::new().route("/users", get(list_user_handler))
        .route("/users/query", get(query_user_handler))
        .with_state(state);
    let listener = TcpListener::bind(addr).await?;
    info!("Listening on {}", addr);
    axum::serve(listener, app.into_make_service()).await?;
    Ok(())
}

#[instrument(skip(state))]
async fn list_user_handler(State(state): State<AppState>) -> Result<impl IntoResponse, StatusCode> {
    let users = state.iter().map(|user| user.value().clone()).collect::<Vec<User>>();
    Ok((StatusCode::OK, Json(users)))
    
}

#[instrument(skip(state))]
async fn query_user_handler(State(state): State<AppState>, Query(filter): Query<UserFilter>) -> impl IntoResponse {
    let users = state.iter().map(|users| users.value().clone()).collect::<Vec<User>>();
    let mut filtered_users = users.into_iter().filter(|user| user.teenager == filter.teenager).collect::<Vec<User>>();
    let limit = filter.limit.clamp(1, i32::MAX) as usize;
    if let Some(order) = filter.order {
        if order == "asc" {
            filtered_users.sort_by(|a, b| a.id.cmp(&b.id));
        } else if order == "desc" {
            filtered_users.sort_by(|a, b| b.id.cmp(&a.id));
        }
    } 
    let limited_users = filtered_users.into_iter().take(limit).collect::<Vec<User>>();
    Json(limited_users)
} 

impl Deref for AppState {
    type Target = Arc<DashMap<i32, User>>;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl AppState {
    fn new() -> Self {
        Self {
            data: Arc::new(DashMap::new()),
        }
    }
}