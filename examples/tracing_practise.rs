
use anyhow::Result;
use tokio::net::TcpListener;
use tracing_subscriber::{
    Layer as _,
    filter::LevelFilter,
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
};
use tracing::{info, instrument};
use axum::{Json, Router, serve};
use axum::routing::get;
#[tokio::main]
async fn main() -> Result<()> {
    let file_appender = tracing_appender::rolling::daily("tmp", "ecosystem.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    let console = fmt::Layer::new()
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .pretty()
        .with_filter(LevelFilter::DEBUG);

    let file = fmt::Layer::new()
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .pretty()
        .with_writer(non_blocking)
        .with_filter(LevelFilter::INFO);

    tracing_subscriber::registry()
        .with(console)
        .with(file)
        .init();

    let addr = "0.0.0.0:8080";
    let app = Router::new().route("/", get(index_handler));
    let listener = TcpListener::bind(addr).await?;
    info!("Listening on {}", addr);
    serve(listener, app.into_make_service()).await?;
    Ok(())
}


#[instrument]// tracing::instument
async fn index_handler() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "message": "Hello, World!"
    }))
}