use std::{collections::HashMap, sync::Arc};

use actix_ws::{Message as WsMessage, Session};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, RwLock};
use validator::Validate;

const MAX_NAME_LENGTH: u64 = 32;
const MAX_MESSAGE_LENGTH: u64 = 2000;

#[derive(Deserialize, Serialize, Validate)]
pub struct Chat {
    #[validate(length(min = 1, max = "MAX_NAME_LENGTH"))]
    name: String,
    #[validate(length(min = 1, max = "MAX_MESSAGE_LENGTH"))]
    message: String,
    time: DateTime<Utc>,
}

impl Chat {
    fn sanitize(&mut self) {
        self.name = self.name.trim().to_string();
        self.message = self.message.trim().to_string();
    }
}

pub async fn handle_incoming_messages(
    mut stream: actix_ws::MessageStream,
    client_id: String,
    tx: broadcast::Sender<String>,
    clients: Arc<RwLock<HashMap<String, Session>>>,
) {
    while let Some(Ok(msg)) = stream.recv().await {
        match msg {
            WsMessage::Close(_reason) => {
                break;
            }
            WsMessage::Ping(bytes) => {
                if let Some(client) = clients.write().await.get_mut(&client_id) {
                    if let Err(err) = client.pong(&bytes).await {
                        eprintln!("Failed to send pong to client {}: {}", client_id, err);
                        break;
                    }
                }
            }
            // Thanks "fyvri"
            WsMessage::Text(text) => match serde_json::from_str::<Chat>(&text) {
                Ok(mut chat) => {
                    chat.sanitize();
                    if chat.validate().is_ok() {
                        if let Err(err) = tx.send(serde_json::to_string(&chat).unwrap()) {
                            eprintln!("Failed to broadcast message: {:?}", err);
                        }
                    }
                }
                Err(e) => eprintln!("Invalid message format: {:?}", e),
            },
            _ => {}
        }
    }

    clients.write().await.remove(&client_id);
}

pub async fn handle_outgoing_messages(
    mut rx: broadcast::Receiver<String>,
    mut session: Session,
    history: Arc<RwLock<Vec<Chat>>>,
) {
    while let Ok(msg) = rx.recv().await {
        if let Ok(chat) = serde_json::from_str::<Chat>(&msg) {
            let json = serde_json::to_string(&chat).unwrap();

            if session.text(json).await.is_err() {
                break;
            }

            history.write().await.push(chat);
        }
    }
}
