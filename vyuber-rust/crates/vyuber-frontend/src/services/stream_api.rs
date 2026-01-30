use gloo_net::http::Request;
use vyuber_shared::stream::StreamKeyResponse;

/// GET /api/stream-key - 既存のストリームキーを取得
pub async fn get_stream_key() -> Result<StreamKeyResponse, String> {
    let response = Request::get("/api/stream-key")
        .send()
        .await
        .map_err(|e| format!("Failed to fetch stream key: {}", e))?;

    if !response.ok() {
        return Err(format!("API error: {}", response.status()));
    }

    response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))
}

/// POST /api/stream-key - 新しいストリームキーを生成
pub async fn generate_stream_key() -> Result<StreamKeyResponse, String> {
    let response = Request::post("/api/stream-key")
        .send()
        .await
        .map_err(|e| format!("Failed to generate stream key: {}", e))?;

    if !response.ok() {
        return Err(format!("API error: {}", response.status()));
    }

    response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))
}
