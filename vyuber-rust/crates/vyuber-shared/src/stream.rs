use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamKey {
    pub key: String,
    pub server_url: String,
    pub full_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamKeyResponse {
    pub stream_key: Option<String>,
    pub server_url: String,
    pub full_url: Option<String>,
}
