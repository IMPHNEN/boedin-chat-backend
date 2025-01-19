use std::sync::Arc;

use tokio::sync::{broadcast, RwLock};

use crate::{models::Chat, CHANNEL_CAPACITY};

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
}
