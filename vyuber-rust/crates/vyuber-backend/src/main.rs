use axum::{
    Router,
    routing::{get, post},
};
use tower_http::{
    cors::CorsLayer,
    services::ServeDir,
};
use std::net::SocketAddr;

mod api;
mod config;
mod services;
mod rtmp;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ログ設定
    tracing_subscriber::fmt::init();

    // 環境変数はInfisical経由で設定されることを想定

    tracing::info!("Starting VYuber Rust Backend...");

    // RTMPサーバーをバックグラウンドで起動
    if let Err(e) = rtmp::start_rtmp_server().await {
        tracing::error!("Failed to start RTMP server: {}", e);
    }

    // 静的ファイルのパスを決定
    let static_path = std::env::var("STATIC_DIR")
        .unwrap_or_else(|_| "crates/vyuber-backend/static".to_string());

    tracing::info!("Serving static files from: {}", static_path);

    // Axum APIルーター
    let app = Router::new()
        // APIルート
        .route("/api/stream-key",
            get(api::stream_key::get_key)
            .post(api::stream_key::generate_key)
            .delete(api::stream_key::delete_key)
        )
        .route("/api/chat", post(api::chat::handle_chat))
        .route("/api/live/:stream_key", get(api::live::stream_flv))
        // 静的ファイル配信 (Leptosビルド成果物)
        .nest_service("/", ServeDir::new(static_path))
        // CORS設定
        .layer(CorsLayer::permissive());

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("Axum server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
