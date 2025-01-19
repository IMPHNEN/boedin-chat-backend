use std::sync::Arc;

use axum::{
    extract::{ws, State, WebSocketUpgrade},
    response::IntoResponse,
};
use chrono::Utc;
use futures::{SinkExt, StreamExt};
use tracing::error;

use crate::{models::Chat, state::AppState};

pub async fn chat_ws(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: ws::WebSocket, state: Arc<AppState>) {
    let (mut ws_tx, mut ws_rx) = socket.split();
    let mut rx = state.tx.subscribe();

    // Handle incoming messages.
    let state_clone = state.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = ws_rx.next().await {
            if let ws::Message::Text(text) = msg {
                if let Ok(mut messages) = serde_json::from_str::<Vec<Chat>>(&text) {
                    for msg in &mut messages {
                        msg.time = Utc::now();

                        state_clone.add_message(msg.clone()).await;

                        if let Err(e) = state_clone.tx.clone().send(msg.clone()) {
                            error!("Failed to send message: {}", e);
                        }
                    }
                } else {
                    error!("Failed to parse message: {}", text);
                }
            }
        }
    });

    // Handle broadcasting and send message history.
    let mut send_task = tokio::spawn(async move {
        // Send message history to the client.
        let recent_messages = state.get_recent_messages().await;
        let json = serde_json::to_string(&recent_messages).unwrap();
        if ws_tx.send(ws::Message::Text(json.into())).await.is_err() {
            return;
        }

        // Broadcast new messages.
        while let Ok(msg) = rx.recv().await {
            let json = serde_json::to_string(&vec![msg]).unwrap();
            if ws_tx.send(ws::Message::Text(json.into())).await.is_err() {
                break;
            }
        }
    });

    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };
}
