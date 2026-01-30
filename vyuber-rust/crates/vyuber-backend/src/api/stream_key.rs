use axum::{Json, http::StatusCode, extract::State};
use uuid::Uuid;
use vyuber_shared::stream::StreamKeyResponse;

use crate::streaming::StreamManager;

/// GET /api/stream-key - 既存のストリームキーを取得
pub async fn get_key(
    State(stream_manager): State<StreamManager>,
) -> Json<StreamKeyResponse> {
    let key = stream_manager.get_active_key().await;
    let rtmp_port = stream_manager.get_rtmp_port().await;
    let server_url = format!("rtmp://localhost:{}/live", rtmp_port);
    Json(StreamKeyResponse {
        stream_key: key,
        server_url,
    })
}

/// POST /api/stream-key - 新しいストリームキーを生成し、FFmpegを(再)起動
pub async fn generate_key(
    State(stream_manager): State<StreamManager>,
) -> Json<StreamKeyResponse> {
    let key = Uuid::new_v4().to_string().replace("-", "");
    let rtmp_port = stream_manager.get_rtmp_port().await;
    let server_url = format!("rtmp://localhost:{}/live", rtmp_port);

    tracing::info!("Generated new stream key: {}", key);

    // ストリームキーを設定 → FFmpeg再起動トリガー
    stream_manager.set_active_key(&key).await;

    Json(StreamKeyResponse {
        stream_key: Some(key),
        server_url,
    })
}

/// DELETE /api/stream-key - ストリームキーを削除
pub async fn delete_key() -> StatusCode {
    tracing::info!("Deleted stream key");
    StatusCode::OK
}
