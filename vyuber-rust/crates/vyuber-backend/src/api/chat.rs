use axum::{Json, http::StatusCode, extract::State};
use vyuber_shared::chat::{ChatComment, ChatRequest, ChatResponse, ChatMode};
use crate::services::groq::GroqClient;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::api::personas::Persona;
use rand::seq::SliceRandom;
use rand::Rng;

#[derive(Clone)]
pub struct ConversationTurn {
    pub streamer_message: String,
    pub viewer_comments: Vec<ChatComment>,
}

#[derive(Clone)]
pub struct ChatHistory {
    pub turns: Arc<Mutex<Vec<ConversationTurn>>>,
    pub main_roster: Vec<Persona>,
    pub gaya_roster: Vec<Persona>,
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
    let last_comments = history.get_last_comments().await;

    // モードごとの設定
    let (active_personas, model_id, system_instruction) = match req.mode {
        ChatMode::Anchor => {
            // ■ 70B (Anchor): 固定ファン
            let mut rng = rand::thread_rng();
            let count = rng.gen_range(3..=5);
            let selected = history.main_roster
                .choose_multiple(&mut rng, count)
                .cloned()
                .collect::<Vec<_>>();
            
            (
                selected, 
                "llama-3.3-70b-versatile", // ★指定されたモデル
                r#"
あなたはYouTubeライブ配信の「固定ファン」です。
提示されたペルソナになりきり、配信者の発言に対して文脈を踏まえたコメントをしてください。
単なる反応だけでなく、質問、感想、ツッコミなど多様な発言を心がけてください。
"#
            )
        },
        ChatMode::Swarm => {
            // ■ 8B (Swarm): ガヤ
            let mut rng = rand::thread_rng();
            let count = rng.gen_range(5..=8);
            let selected = history.gaya_roster
                .choose_multiple(&mut rng, count)
                .cloned()
                .collect::<Vec<_>>();

            (
                selected,
                "llama-3.1-8b-instant", // 8B固定
                r#"
あなたはライブ配信の「ガヤ」です。
配信者の言葉に対し、反射的に短いリアクションだけを返してください。
長い文章は禁止です。「ｗｗｗ」「草」「８８８８」「なるほど」「！？」などの短文のみ許可します。
"#
            )
        }
    };

    match client.generate_comments_with_context(
        &req.message, 
        last_comments.as_deref(), 
        &active_personas,
        model_id,
        system_instruction
    ).await {
        Ok(comments) => {
            // 履歴保存はAnchorのみ（文脈維持のため）
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