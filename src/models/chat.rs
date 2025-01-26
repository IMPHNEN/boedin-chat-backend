use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

#[derive(Clone, Deserialize, Serialize, FromRow)]
pub struct Chat {
    pub name: String,
    pub content: String,
    pub timestamp: String,
}
