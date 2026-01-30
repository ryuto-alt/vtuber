use axum::{
    Router,
    routing::{get, post},
};
use tower_http::{
    cors::CorsLayer,
    services::ServeDir,
};
use std::net::SocketAddr;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

mod api;
mod config;
mod services;
mod rtmp;
mod streaming;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ログをファイルとコンソール両方に出力
    let log_dir = std::env::var("LOG_DIR").unwrap_or_else(|_| "logs".to_string());
    std::fs::create_dir_all(&log_dir).ok();
    let file_appender = tracing_appender::rolling::daily(&log_dir, "vyuber.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(fmt::layer().with_ansi(true)) // コンソール
        .with(fmt::layer().with_ansi(false).with_writer(non_blocking)) // ファイル
        .init();

    tracing::info!("Starting VYuber Rust Backend...");

    // StreamManagerを初期化
    let stream_manager = streaming::StreamManager::new();

    // RTMPサーバー（FFmpegリスナー）をバックグラウンドで起動
    if let Err(e) = rtmp::start_rtmp_server(stream_manager.clone()).await {
        tracing::error!("Failed to start RTMP server: {}", e);
    }

    let static_path = std::env::var("STATIC_DIR")
        .unwrap_or_else(|_| "crates/vyuber-backend/static".to_string());

    tracing::info!("Serving static files from: {}", static_path);

    let app = Router::new()
        .route("/api/stream-key",
            get(api::stream_key::get_key)
            .post(api::stream_key::generate_key)
            .delete(api::stream_key::delete_key)
        )
        .route("/api/chat", post(api::chat::handle_chat))
        .route("/api/live/stream", get(api::live::stream_flv))
        .nest_service("/", ServeDir::new(static_path))
        .layer(CorsLayer::permissive())
        .with_state(stream_manager);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("Axum server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
