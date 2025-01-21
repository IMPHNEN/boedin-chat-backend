use std::sync::Arc;

use tokio::sync::{broadcast, RwLock};

use crate::{models::Chat, CHANNEL_CAPACITY, HISTORY_LIMIT};

pub struct AppState {
    pub tx: broadcast::Sender<Chat>,
    pub history: Arc<RwLock<Vec<Chat>>>,
}

impl AppState {
    pub fn new() -> Arc<Self> {
        let (tx, _) = broadcast::channel::<Chat>(CHANNEL_CAPACITY);

        Arc::new(Self {
            tx,
            history: Arc::new(RwLock::new(Vec::new())),
        })
    }

    pub async fn add_message(&self, message: Chat) {
        let mut history = self.history.write().await;

        history.push(message);

        if history.len() > HISTORY_LIMIT {
            history.remove(0);
        }
    }

    pub async fn get_recent_messages(&self) -> Vec<Chat> {
        self.history.read().await.iter().cloned().collect()
    }
}
