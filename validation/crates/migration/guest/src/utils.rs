use serde::{Deserialize, Serialize};
use tlsn_core::ServerName;

#[derive(Serialize, Deserialize)]
pub struct Input {
    pub transcript: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Output {
    pub server_name: ServerName,
    pub time: chrono::DateTime<chrono::Utc>,
    pub sent_data: String,
    pub received_data: String,
}
