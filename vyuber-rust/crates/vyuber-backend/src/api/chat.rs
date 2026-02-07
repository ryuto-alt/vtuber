use axum::{Json, http::StatusCode, extract::State};
use serde::{Deserialize, Serialize};
use vyuber_shared::chat::ChatComment;
use crate::services::groq::GroqClient;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::api::personas::Persona;
use rand::seq::SliceRandom;
use rand::Rng; // 追加: 乱数生成用

#[derive(Clone)]
pub struct ConversationTurn {
    pub streamer_message: String,
    pub viewer_comments: Vec<ChatComment>,
}

#[derive(Clone)]
pub struct ChatHistory {
    pub turns: Arc<Mutex<Vec<ConversationTurn>>>,
    pub main_roster: Vec<Persona>, // メイン層
    pub gaya_roster: Vec<Persona>, // ガヤ層
}

impl ChatHistory {
    pub fn new() -> Self {
        Self {
            turns: Arc::new(Mutex::new(Vec::with_capacity(3))),
            main_roster: Persona::create_main_roster(),
            gaya_roster: Persona::create_gaya_roster(),
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

/// POST /api/chat
pub async fn handle_chat(
    State(history): State<ChatHistory>,
    Json(req): Json<ChatRequest>,
) -> Result<Json<ChatResponse>, (StatusCode, Json<ErrorResponse>)> {
    tracing::info!("[Chat API] Received message: {}", req.message);

    // ★修正: メイン5人 + ガヤ数人を選出する
    let active_personas: Vec<Persona> = {
        let mut rng = rand::thread_rng();
        
        // 1. メイン層から必ず5人選ぶ
        let mut selected = history.main_roster
            .choose_multiple(&mut rng, 5)
            .cloned()
            .collect::<Vec<_>>();

        // 2. ガヤ層からランダムに 2〜5人 選ぶ
        let gaya_count = rng.gen_range(2..=5);
        let gaya = history.gaya_roster
            .choose_multiple(&mut rng, gaya_count)
            .cloned();
        
        selected.extend(gaya);
        selected
    };
    
    // デバッグログ
    let names: Vec<&str> = active_personas.iter().map(|p| p.name.as_str()).collect();
    tracing::info!("[Chat API] Selected personas (Total {}): {:?}", active_personas.len(), names);

    let client = GroqClient::from_env();
    let last_comments = history.get_last_comments().await;

    match client.generate_comments_with_context(&req.message, last_comments.as_deref(), &active_personas).await {
        Ok(comments) => {
            history.add_turn(req.message, comments.clone()).await;
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