use axum::{Json, http::StatusCode, extract::State};
use serde::{Deserialize, Serialize};
use vyuber_shared::chat::ChatComment;
use crate::services::groq::GroqClient;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::VecDeque;

/// 会話履歴を保持する構造体（直近5件の配信者発言）
#[derive(Clone, Default)]
pub struct ChatHistory {
    pub messages: Arc<Mutex<VecDeque<String>>>,
}

impl ChatHistory {
    pub fn new() -> Self {
        Self {
            messages: Arc::new(Mutex::new(VecDeque::with_capacity(5))),
        }
    }

    /// 新しいメッセージを追加（最大5件保持）
    pub async fn add_message(&self, message: String) {
        let mut messages = self.messages.lock().await;
        if messages.len() >= 5 {
            messages.pop_front();
        }
        messages.push_back(message);
    }

    /// 履歴を取得
    pub async fn get_history(&self) -> Vec<String> {
        let messages = self.messages.lock().await;
        messages.iter().cloned().collect()
    }
}

#[derive(Deserialize)]
pub struct ChatRequest {
    pub message: String,
}

#[derive(Serialize)]
pub struct ChatResponse {
    pub comments: Vec<ChatComment>,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub details: Option<String>,
}

/// POST /api/chat - Groq APIを使ってコメントを生成
pub async fn handle_chat(
    State(history): State<ChatHistory>,
    Json(req): Json<ChatRequest>,
) -> Result<Json<ChatResponse>, (StatusCode, Json<ErrorResponse>)> {
    tracing::info!("[Chat API] Received message: {}", req.message);

    // 過去の会話履歴を取得
    let past_messages = history.get_history().await;
    tracing::info!("[Chat API] History count: {}", past_messages.len());

    // Groq APIクライアントを作成
    let client = GroqClient::from_env();

    // コメントを生成（履歴付き）
    match client.generate_comments_with_history(&req.message, &past_messages).await {
        Ok(comments) => {
            // 成功したら履歴に追加
            history.add_message(req.message).await;
            tracing::info!("[Chat API] Successfully generated {} comments", comments.len());
            Ok(Json(ChatResponse { comments }))
        }
        Err(e) => {
            tracing::error!("[Chat API] Error details: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "API Error".to_string(),
                    details: Some(e.to_string()),
                }),
            ))
        }
    }
}
