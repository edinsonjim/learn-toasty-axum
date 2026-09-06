use std::net::SocketAddr;
use std::sync::Arc;

use axum::{Router, routing::get};

mod error;
mod handlers;
mod models;

#[derive(Clone)]
pub struct AppState {
    db: Arc<tokio::sync::Mutex<toasty::Db>>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let db = toasty::Db::builder()
        .models(toasty::models!(models::Family))
        .connect("sqlite:./families.db")
        .await?;

    db.push_schema().await?;

    let state = AppState {
        db: Arc::new(tokio::sync::Mutex::new(db)),
    };

    let app = Router::new()
        .route("/", get(handlers::health))
        .route(
            "/families",
            axum::routing::get(handlers::list_families).post(handlers::create_family),
        )
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("listening on http://{addr}");
    axum::serve(listener, app).await?;

    Ok(())
}
