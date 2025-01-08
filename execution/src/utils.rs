use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize)]
pub struct Input {
    pub transcript: String,
}

