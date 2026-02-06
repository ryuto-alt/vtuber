use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamMetadata {
    pub session_id: String,
    pub title: String,
    pub started_at: String, // ISO 8601
    pub ended_at: String,
    pub duration_seconds: u32,
    pub total_messages: usize,
    pub ai_viewer_count: usize,
    pub recording_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamMetadataList {
    pub sessions: Vec<StreamMetadata>,
}
