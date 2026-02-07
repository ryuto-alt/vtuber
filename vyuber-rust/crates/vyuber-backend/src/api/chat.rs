use axum::{Json, http::StatusCode, extract::State};
use serde::{Deserialize, Serialize};
use vyuber_shared::chat::ChatComment;
use crate::services::groq::GroqClient;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::api::personas::Persona;
use rand::seq::SliceRandom;

/// 1ターンの会話（配信者の発言 + 視聴者のコメント）
#[derive(Clone)]
pub struct ConversationTurn {
    pub streamer_message: String,
    pub viewer_comments: Vec<ChatComment>,
}

/// 会話履歴を保持する構造体
#[derive(Clone)]
pub struct ChatHistory {
    pub turns: Arc<Mutex<Vec<ConversationTurn>>>,
    pub roster: Vec<Persona>,
}

impl ChatHistory {
    pub fn new() -> Self {
        Self {
            turns: Arc::new(Mutex::new(Vec::with_capacity(3))),
            roster: Persona::create_roster(),
        }
    }

    /// 新しいターンを追加
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

    // ★修正ポイント: ブロック { } で囲んで rng の寿命をここで終わらせる
    let active_personas: Vec<Persona> = {
        let mut rng = rand::thread_rng();
        let count = match req.message.chars().count() {
            0..=5 => 2,   // 短い言葉なら2人
            6..=20 => 3,  // 普通なら3人
            _ => 4,       // 長文なら4人で盛り上げる
        };

        history.roster
            .choose_multiple(&mut rng, count)
            .cloned()
            .collect()
    }; // <--- ここで rng は消滅する！これで安全に await できる。
    
    // 誰が選ばれたかログに出す
    let names: Vec<&str> = active_personas.iter().map(|p| p.name.as_str()).collect();
    tracing::info!("[Chat API] Selected personas: {:?}", names);

    // Groq APIクライアントを作成
    let client = GroqClient::from_env();
    let last_comments = history.get_last_comments().await;

    // 選抜メンバーを渡してコメント生成
    match client.generate_comments_with_context(&req.message, last_comments.as_deref(), &active_personas).await {
        Ok(comments) => {
            // 成功したら履歴に追加
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