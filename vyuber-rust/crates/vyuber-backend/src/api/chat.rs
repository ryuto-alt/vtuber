use axum::{Json, http::StatusCode, extract::State};
use serde::{Deserialize, Serialize};
use vyuber_shared::chat::ChatComment;
use crate::services::groq::GroqClient;
use std::sync::Arc;
use tokio::sync::Mutex;

/// 1ターンの会話（配信者の発言 + 視聴者のコメント）
#[derive(Clone)]
pub struct ConversationTurn {
    pub streamer_message: String,
    pub viewer_comments: Vec<ChatComment>,
}

/// 会話履歴を保持する構造体（直近3ターン）
#[derive(Clone, Default)]
pub struct ChatHistory {
    pub turns: Arc<Mutex<Vec<ConversationTurn>>>,
}

impl ChatHistory {
    pub fn new() -> Self {
        Self {
            turns: Arc::new(Mutex::new(Vec::with_capacity(3))),
        }
    }

    /// 新しいターンを追加（最大3件保持）
    pub async fn add_turn(&self, streamer_message: String, viewer_comments: Vec<ChatComment>) {
        let mut turns = self.turns.lock().await;
        if turns.len() >= 3 {
            turns.remove(0);
        }
        turns.push(ConversationTurn {
            streamer_message,
            viewer_comments,
        });
    }

    /// 直前のターンの視聴者コメントを取得
    pub async fn get_last_comments(&self) -> Option<Vec<ChatComment>> {
        let turns = self.turns.lock().await;
        turns.last().map(|t| t.viewer_comments.clone())
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

    // 前のターンの視聴者コメントを取得
    let last_comments = history.get_last_comments().await;
    tracing::info!("[Chat API] Has previous comments: {}", last_comments.is_some());

    // Groq APIクライアントを作成
    let client = GroqClient::from_env();

    // コメントを生成（前のコメント付き）
    match client.generate_comments_with_context(&req.message, last_comments.as_deref()).await {
        Ok(comments) => {
            // 成功したら履歴に追加（配信者の発言と視聴者コメント）
            history.add_turn(req.message, comments.clone()).await;
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
