use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: i64,
    pub user: String,
    pub text: String,
    pub color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatComment {
    pub user: String,
    pub text: String,
    pub color: String,
}

// ★追加: チャットのモード指定
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChatMode {
    Anchor, // 70B (固定ファン)
    Swarm,  // 8B (ガヤ)
}

#[derive(Serialize, Deserialize)]
pub struct ChatRequest {
    pub message: String,
    pub mode: ChatMode, // ★追加
}

#[derive(Serialize, Deserialize)]
pub struct ChatResponse {
    pub comments: Vec<ChatComment>,
}