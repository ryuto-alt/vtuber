use axum::{Json, http::StatusCode};
use once_cell::sync::Lazy;
use std::sync::{Arc, RwLock};
use uuid::Uuid;
use vyuber_shared::stream::StreamKeyResponse;

// グローバルなストリームキー管理（メモリベース）
static STREAM_KEY: Lazy<Arc<RwLock<Option<String>>>> = Lazy::new(|| {
    Arc::new(RwLock::new(None))
});

/// GET /api/stream-key - 既存のストリームキーを取得
pub async fn get_key() -> Json<StreamKeyResponse> {
    let key = STREAM_KEY.read().unwrap().clone();
    let rtmp_port = std::env::var("RTMP_PORT").unwrap_or("1935".to_string());
    let server_url = format!("rtmp://localhost:{}/live", rtmp_port);
    let full_url = key.as_ref().map(|k| format!("{}/{}", server_url, k));

    Json(StreamKeyResponse {
        stream_key: key,
        server_url,
        full_url,
    })
}

/// POST /api/stream-key - 新しいストリームキーを生成
pub async fn generate_key() -> Json<StreamKeyResponse> {
    let key = Uuid::new_v4().to_string().replace("-", "");
    let rtmp_port = std::env::var("RTMP_PORT").unwrap_or("1935".to_string());
    let server_url = format!("rtmp://localhost:{}/live", rtmp_port);
    let full_url = format!("{}/{}", server_url, key);

    *STREAM_KEY.write().unwrap() = Some(key.clone());

    tracing::info!("Generated new stream key: {}", key);

    Json(StreamKeyResponse {
        stream_key: Some(key),
        server_url,
        full_url: Some(full_url),
    })
}

/// DELETE /api/stream-key - ストリームキーを削除
pub async fn delete_key() -> StatusCode {
    *STREAM_KEY.write().unwrap() = None;
    tracing::info!("Deleted stream key");
    StatusCode::OK
}
