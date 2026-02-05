use axum::{
    Router,
    routing::{get, post},
};
use tower_http::{
    cors::CorsLayer,
    services::ServeDir,
};
use std::net::SocketAddr;

pub mod api;
pub mod config;
pub mod services;
pub mod streaming;
pub mod mediamtx;

use api::chat::ChatHistory;

/// Axumサーバーを起動する（トレーシング設定は呼び出し側の責務）
pub async fn start_server() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing::info!("Starting VYuber Rust Backend...");

    // MediaMTXをバックグラウンドで起動
    if let Err(e) = mediamtx::start_mediamtx().await {
        tracing::error!("Failed to start MediaMTX: {}", e);
        tracing::warn!("Continuing without MediaMTX — manual startup required");
    }

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

    let app = Router::new()
        .route("/api/stream-key",
            get(api::stream_key::get_key)
            .post(api::stream_key::generate_key)
            .delete(api::stream_key::delete_key)
        )
        .merge(chat_router)
        // 音声ファイル送信ルート（既存）
        .route("/api/transcribe", post(api::deepgram::transcribe))
        // リアルタイム音声認識ルート
        .route("/api/transcribe/live", get(services::deepgram_stream::handler))
        .route("/api/live/status", get(api::live::stream_status))
        .route("/api/live/whep", post(api::live::whep_proxy))
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
