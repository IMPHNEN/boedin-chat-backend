use std::net::SocketAddr;

use axum::{routing, Router};
use tokio::net::TcpListener;
use tracing::info;

use routes::chat_ws;
use state::AppState;

mod models;
mod routes;
mod state;

const HISTORY_LIMIT: usize = 100;
const CHANNEL_CAPACITY: usize = 1000;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app_state = AppState::new();

    let app = Router::new()
        .route("/ws", routing::any(chat_ws))
        .with_state(app_state.clone());

    let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();

    info!("Web Server running on {}", listener.local_addr().unwrap());
    info!("Chat Server running on {}", listener.local_addr().unwrap());

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}
