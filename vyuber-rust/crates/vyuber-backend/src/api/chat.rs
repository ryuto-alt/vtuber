use axum::{Json, http::StatusCode, extract::State};
use vyuber_shared::chat::{ChatComment, ChatRequest, ChatResponse, ChatMode};
use crate::services::groq::GroqClient;
use crate::orchestrator::Orchestrator;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct ConversationTurn {
    pub streamer_message: String,
    pub viewer_comments: Vec<ChatComment>,
}

#[derive(Clone)]
pub struct ChatHistory {
    pub turns: Arc<Mutex<Vec<ConversationTurn>>>,
    // ロスター管理は Orchestrator に移動したため削除
}

impl ChatHistory {
    pub fn new() -> Self {
        Self {
            turns: Arc::new(Mutex::new(Vec::with_capacity(3))),
        }
    }

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

    pub async fn get_last_comments(&self) -> Option<Vec<ChatComment>> {
        let turns = self.turns.lock().await;
        turns.last().map(|t| t.viewer_comments.clone())
    }
}

#[derive(serde::Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub details: Option<String>,
}

pub async fn handle_chat(
    State(history): State<ChatHistory>,
    Json(req): Json<ChatRequest>,
) -> Result<Json<ChatResponse>, (StatusCode, Json<ErrorResponse>)> {
    tracing::info!("[Chat API] Mode: {:?}, Message: {}", req.mode, req.message);

    let client = GroqClient::from_env();
    
    // オーケストレーターを初期化（ここでペルソナリストも生成される）
    let orchestrator = Orchestrator::new(); 
    
    let last_comments = history.get_last_comments().await;

    // Orchestratorに選抜と設定を委譲
    let (active_personas, model_id, system_instruction) = orchestrator.dispatch(req.mode.clone());

    match client.generate_comments_with_context(
        &req.message, 
        last_comments.as_deref(), 
        &active_personas,
        model_id,
        system_instruction
    ).await {
        Ok(comments) => {
            // Anchorモード（70B）の時だけ文脈履歴を更新する
            if req.mode == ChatMode::Anchor {
                history.add_turn(req.message.clone(), comments.clone()).await;
            }
            Ok(Json(ChatResponse { comments }))
        }
        Err(e) => {
            tracing::error!("[Chat API] Error: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse { error: "API Error".to_string(), details: Some(e.to_string()) }),
            ))
        }
    }
}