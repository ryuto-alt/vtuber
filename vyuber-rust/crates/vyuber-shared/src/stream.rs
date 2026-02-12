use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamKeyResponse {
    pub stream_key: Option<String>,
    pub server_url: String,
}
