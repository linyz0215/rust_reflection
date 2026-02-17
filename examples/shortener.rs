use anyhow::Result;
use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderMap, StatusCode, header::LOCATION},
    response::IntoResponse,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use tokio::net::TcpListener;
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{Layer as _, fmt::Layer, layer::SubscriberExt, util::SubscriberInitExt};
use nanoid::nanoid;

const addr: &str = "127.0.0.1:8080";
#[derive(Clone)]
struct AppState {
    pool: PgPool,
}

#[derive(Deserialize)]
struct ShortenRequest {
    url: String,
}

#[derive(Serialize)]
struct ShortenResponse {
    url: String,
}

#[derive(FromRow)]
struct urlRecord {
    #[sqlx(default)]
    id: String,
    #[sqlx(default)]
    url: String,
}
#[tokio::main]
async fn main() -> Result<()> {
    let layer = Layer::new().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();

    let listener = TcpListener::bind(addr).await?;
    info!("Listening on {}", addr);
    let db_url: &str = "postgres://linyz@localhost/shortener1";
    let state: AppState = AppState::try_new(db_url).await?;
    let app: Router = Router::new()
        .route("/route", post(shorten_handler))
        .route("/redirect/{id}", get(redirect_handler))
        .with_state(state.clone());
    axum::serve(listener, app.into_make_service()).await?;
    Ok(())
}

async fn shorten_handler(
    State(state): State<AppState>,
    Json(req): Json<ShortenRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let id = state.shorten(req).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let res = ShortenResponse {
        url: format!("http://{}/redirect/{}", addr, id),
    };
    let body = Json(res);
    Ok((StatusCode::OK, body))
}

async fn redirect_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, StatusCode> {
    let url = state.get_url(id).await.map_err(|_| StatusCode::NOT_FOUND)?;
    let mut header = HeaderMap::new();
    header.insert(LOCATION, url.parse().unwrap());
    Ok((StatusCode::FOUND, header))
}


impl AppState {
    async fn try_new(url: &str) -> Result<Self> {
        let pool = PgPool::connect(url).await?;
        let id = nanoid!(6);
        sqlx::query( 
        r#"CREATE TABLE IF NOT EXISTS urls (
        url TEXT NOT NULL PRIMARY KEY,
        id TEXT NOT NULL
        )"#
        )
        .execute(&pool)
        .await?;
        Ok(Self {
            pool,
        })
    }


    async fn shorten(&self, req: ShortenRequest) -> Result<String> {
        let id = nanoid!(6);
        let url: urlRecord = sqlx::query_as(
            r#"
                INSERT INTO urls (url, id) VALUES ($1, $2) ON CONFLICT(url) DO UPDATE SET url = EXCLUDED.url RETURNING id
            "#
        )
        .bind(req.url)
        .bind(id)
        .fetch_one(&self.pool)
        .await?;
        Ok(url.id)
    }


    async fn get_url(&self, id: String) -> Result<String> {
        let res: urlRecord = sqlx::query_as(
            r#"
                SELECT url FROM urls WHERE id = $1
            "#
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;
        Ok(res.url)
    }
}