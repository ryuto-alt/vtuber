use axum::{Json, http::StatusCode};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct RecordingRequest {
    pub action: String, // "start" or "stop"
    pub stream_key: String,
}

#[derive(Serialize)]
pub struct RecordingResponse {
    pub success: bool,
    pub message: String,
}

pub async fn control_recording(
    Json(req): Json<RecordingRequest>,
) -> (StatusCode, Json<RecordingResponse>) {
    let url = format!("http://127.0.0.1:9997/v3/config/paths/patch/{}", req.stream_key);

    let record_value = match req.action.as_str() {
        "start" => "yes",
        "stop" => "no",
        _ => return (StatusCode::BAD_REQUEST, Json(RecordingResponse {
            success: false,
            message: "Invalid action".to_string(),
        })),
    };

    let body = serde_json::json!({ "record": record_value });

    match reqwest::Client::new().patch(&url).json(&body).send().await {
        Ok(_) => (StatusCode::OK, Json(RecordingResponse {
            success: true,
            message: format!("Recording {} successful", req.action),
        })),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(RecordingResponse {
            success: false,
            message: format!("Failed: {}", e),
        })),
    }
}
