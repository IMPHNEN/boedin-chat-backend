use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize)]
pub struct Chat {
    name: String,
    message: String,
    pub time: DateTime<Utc>,
}
