use axum::{
    extract::{Multipart, State},
    response::Json,
};
use serde_json::{json, Value};
use std::sync::Arc;

use crate::services::whisper::WhisperService;

pub async fn transcribe(
    State(whisper): State<Arc<WhisperService>>,
    mut multipart: Multipart,
) -> Json<Value> {
    while let Some(field) = multipart.next_field().await.unwrap_or(None) {
        if field.name() == Some("audio") {
            let data = match field.bytes().await {
                Ok(bytes) => bytes,
                Err(e) => {
                    return Json(
                        json!({ "status": "error", "message": format!("Failed to read bytes: {}", e) }),
                    )
                }
            };

            // PCM f32 (16kHz mono) として解釈
            let samples: Vec<f32> = data
                .chunks_exact(4)
                .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
                .collect();

            let whisper_clone = whisper.clone();
            let result =
                tokio::task::spawn_blocking(move || whisper_clone.transcribe(&samples)).await;

            match result {
                Ok(Ok(text)) => {
                    tracing::info!("音声認識成功: {}", text);
                    return Json(json!({ "status": "success", "text": text }));
                }
                Ok(Err(e)) => {
                    tracing::error!("音声認識エラー: {}", e);
                    return Json(json!({ "status": "error", "message": e }));
                }
                Err(e) => {
                    tracing::error!("Task join error: {}", e);
                    return Json(
                        json!({ "status": "error", "message": format!("Internal error: {}", e) }),
                    );
                }
            }
        }
    }

    Json(json!({ "status": "error", "message": "No audio file found" }))
}
