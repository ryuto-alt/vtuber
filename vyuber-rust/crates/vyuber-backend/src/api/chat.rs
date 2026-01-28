use axum::{Json, http::StatusCode};
use serde::{Deserialize, Serialize};
use vyuber_shared::chat::ChatComment;
use crate::services::gemini::GeminiClient;

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

/// POST /api/chat - Gemini APIを使ってコメントを生成
pub async fn handle_chat(
    Json(req): Json<ChatRequest>,
) -> Result<Json<ChatResponse>, (StatusCode, Json<ErrorResponse>)> {
    tracing::info!("[Chat API] Received message: {}", req.message);

    // Gemini APIクライアントを作成
    let client = GeminiClient::from_env();

    // コメントを生成
    match client.generate_comments(&req.message).await {
        Ok(comments) => {
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
