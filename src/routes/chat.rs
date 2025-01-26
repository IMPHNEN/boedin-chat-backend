use std::sync::Arc;

use axum::{
    extract::{ws::Message, State, WebSocketUpgrade},
    response::IntoResponse,
};
use futures_util::{SinkExt, StreamExt};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde_json::Value;
use tokio::sync::Mutex;
use tracing::error;

use crate::{
    models::{Chat, Claims},
    states::AppState,
    utils::JWT_SECRET,
};

pub async fn chat_ws(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| async move {
        let (sender, mut receiver) = socket.split();
        let sender = Arc::new(Mutex::new(sender));

        if let Some(Ok(Message::Text(token_message))) = receiver.next().await {
            if let Ok(token) = serde_json::from_str::<Value>(&token_message) {
                if let Some(token) = token.get("token").and_then(|t| t.as_str()) {
                    let validation = Validation::new(Algorithm::HS256);
                    let token_data = decode::<Claims>(
                        token,
                        &DecodingKey::from_secret(JWT_SECRET.as_bytes()),
                        &validation,
                    );

                    if token_data.is_ok() {
                        let history = state.get_history().await;
                        for chat in history {
                            if let Err(e) = sender
                                .clone()
                                .lock()
                                .await
                                .send(Message::Text(serde_json::to_string(&chat).unwrap().into()))
                                .await
                            {
                                error!("Failed to send chat history: {}", e);
                                break;
                            }
                        }

                        let state_clone = state.clone();
                        let sender_clone = sender.clone();

                        let mut send_task = tokio::spawn(async move {
                            while let Ok(chat) = state_clone.sender.subscribe().recv().await {
                                if let Err(e) = sender_clone
                                    .clone()
                                    .lock()
                                    .await
                                    .send(Message::Text(
                                        serde_json::to_string(&chat).unwrap().into(),
                                    ))
                                    .await
                                {
                                    error!("Failed to send broadcast message: {}", e);
                                    break;
                                }
                            }
                        });

                        let mut recv_task = tokio::spawn(async move {
                            while let Some(Ok(msg)) = receiver.next().await {
                                if let Message::Text(text) = msg {
                                    if let Ok(chat) = serde_json::from_str::<Chat>(&text) {
                                        state.add_message(chat).await;
                                    } else {
                                        error!(
                                            "Failed to parse incoming message as JSON: {}",
                                            text
                                        );
                                    }
                                }
                            }
                        });

                        tokio::select! {
                            _ = (&mut send_task) => recv_task.abort(),
                            _ = (&mut recv_task) => send_task.abort()
                        }
                    } else {
                        error!("Invalid JWT token");
                    }
                }
            }
        }

        let _ = sender.lock().await.close().await;
    })
}
