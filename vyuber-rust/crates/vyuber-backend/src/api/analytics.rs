use axum::{Json, http::StatusCode};
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;
use vyuber_shared::analytics::{StreamMetadata, StreamMetadataList};

const METADATA_FILE: &str = "./data/stream_metadata.json";

#[derive(Deserialize)]
pub struct SaveMetadataRequest {
    pub metadata: StreamMetadata,
}

pub async fn save_metadata(
    Json(req): Json<SaveMetadataRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    if let Some(parent) = PathBuf::from(METADATA_FILE).parent() {
        let _ = fs::create_dir_all(parent);
    }

    let mut list = match fs::read_to_string(METADATA_FILE) {
        Ok(content) => serde_json::from_str::<StreamMetadataList>(&content)
            .unwrap_or(StreamMetadataList { sessions: vec![] }),
        Err(_) => StreamMetadataList { sessions: vec![] },
    };

    list.sessions.push(req.metadata);

    match fs::write(METADATA_FILE, serde_json::to_string_pretty(&list).unwrap()) {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({
            "success": true,
            "message": "Metadata saved"
        }))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
            "success": false,
            "message": format!("Failed to save: {}", e)
        }))),
    }
}

pub async fn list_metadata() -> (StatusCode, Json<StreamMetadataList>) {
    match fs::read_to_string(METADATA_FILE) {
        Ok(content) => {
            let list = serde_json::from_str::<StreamMetadataList>(&content)
                .unwrap_or(StreamMetadataList { sessions: vec![] });
            (StatusCode::OK, Json(list))
        },
        Err(_) => (StatusCode::OK, Json(StreamMetadataList { sessions: vec![] })),
    }
}
