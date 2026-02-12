use axum::{
    Router,
    routing::{get, post},
};
use tower_http::{
    cors::CorsLayer,
    services::ServeDir,
};
use std::net::SocketAddr;
use std::sync::Arc;

pub mod api;
pub mod config;
pub mod services;
pub mod streaming;
pub mod mediamtx;

use api::chat::ChatHistory;
use services::whisper::WhisperService;

/// Axumサーバーを起動する（トレーシング設定は呼び出し側の責務）
pub async fn start_server() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing::info!("Starting VYuber Rust Backend...");

    // MediaMTXをバックグラウンドで起動
    if let Err(e) = mediamtx::start_mediamtx().await {
        tracing::error!("Failed to start MediaMTX: {}", e);
        tracing::warn!("Continuing without MediaMTX — manual startup required");
    }

    // Whisperモデルをロード
    let model_path = std::env::var("WHISPER_MODEL_PATH")
        .unwrap_or_else(|_| "models/ggml-medium.bin".to_string());
    let whisper_service = Arc::new(
        WhisperService::new(&model_path)
            .expect("Failed to load Whisper model. Run download-model.bat first.")
    );

    // StreamManagerを初期化
    let stream_manager = streaming::StreamManager::new();
    // 会話履歴を初期化
    let chat_history = ChatHistory::new();

    let static_path = std::env::var("STATIC_DIR")
        .unwrap_or_else(|_| "crates/vyuber-backend/static".to_string());

    tracing::info!("Serving static files from: {}", static_path);

    // チャット用のルーター（ChatHistoryをStateとして使用）
    let chat_router = Router::new()
        .route("/api/chat", post(api::chat::handle_chat))
        .with_state(chat_history);

    // 音声認識用のルーター（WhisperServiceをStateとして使用）
    let whisper_router = Router::new()
        .route("/api/transcribe", post(api::transcribe::transcribe))
        .route("/api/transcribe/live", get(services::whisper_stream::handler))
        .with_state(whisper_service);

    let app = Router::new()
        .route("/api/stream-key",
            get(api::stream_key::get_key)
            .post(api::stream_key::generate_key)
            .delete(api::stream_key::delete_key)
        )
        .merge(chat_router)
        .merge(whisper_router)
        .route("/api/live/status", get(api::live::stream_status))
        .route("/api/live/whep", post(api::live::whep_proxy))
        // 分析API
        .route("/api/analytics/save", post(api::analytics::save_metadata))
        .route("/api/analytics/list", get(api::analytics::list_metadata))
        // 録画API
        .route("/api/recording", post(api::recording::control_recording))
        .nest_service("/", ServeDir::new(static_path))
        .layer(CorsLayer::permissive())
        .with_state(stream_manager);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("Axum server listening on {}", addr);
    tracing::info!("MediaMTX WebRTC: http://localhost:8889/");
    tracing::info!("MediaMTX RTMP: rtmp://localhost:1935/live/{{stream_key}}");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
