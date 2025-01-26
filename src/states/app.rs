use std::{collections::VecDeque, sync::Arc};

use sqlx::{query, query_as, SqlitePool};
use tokio::sync::{broadcast, RwLock};
use tracing::error;

use crate::{models::Chat, utils::{CHANNEL_CAPACITY, HISTORY_LIMIT}};

pub struct AppState {
    pub pool: SqlitePool,
    pub sender: broadcast::Sender<Chat>,
    pub history: Arc<RwLock<VecDeque<Chat>>>,
    pub code_verifier: Arc<RwLock<Option<String>>>,
}

impl AppState {
    pub async fn new(pool: SqlitePool) -> Arc<Self> {
        let (sender, _) = broadcast::channel(*CHANNEL_CAPACITY);

        let state = Arc::new(Self {
            pool,
            sender,
            history: Arc::new(RwLock::new(VecDeque::with_capacity(*HISTORY_LIMIT))),
            code_verifier: Arc::new(RwLock::new(None)),
        });

        state.load_history().await;
        state
    }

    pub async fn load_history(&self) {
        let result = query_as::<_, Chat>(
            "SELECT name, content, timestamp FROM messages ORDER BY timestamp ASC",
        )
        .fetch_all(&self.pool)
        .await;

        match result {
            Ok(messages) => {
                let mut history = self.history.write().await;
                for message in messages {
                    if history.len() >= history.capacity() {
                        history.pop_front();
                    }

                    history.push_back(message);
                }
            }
            Err(e) => {
                error!("Failed to load chat history from database: {}", e);
            }
        }
    }

    pub async fn add_message(&self, chat: Chat) {
        let mut history = self.history.write().await;

        let result = query("INSERT INTO messages (name, content, timestamp) VALUES (?, ?, ?)")
            .bind(&chat.name)
            .bind(&chat.content)
            .bind(&chat.timestamp)
            .execute(&self.pool)
            .await;

        if let Err(e) = result {
            error!("Failed to save message to database: {}", e);
            return;
        }

        if history.len() >= history.capacity() {
            history.pop_front();
        }

        history.push_back(chat.clone());

        if let Err(e) = self.sender.send(chat) {
            error!("Failed to send broadcast message: {}", e);
        }
    }

    pub async fn get_history(&self) -> Vec<Chat> {
        self.history.read().await.iter().cloned().collect()
    }

    pub async fn save_user(
        &self,
        user_id: &str,
        username: &str,
        role: &str,
    ) -> Result<(), sqlx::Error> {
        query("INSERT OR REPLACE INTO users (id, username, role) VALUES (?, ?, ?)")
            .bind(user_id)
            .bind(username)
            .bind(role)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}
