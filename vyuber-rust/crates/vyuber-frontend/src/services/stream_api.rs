use gloo_net::http::Request;
use vyuber_shared::stream::StreamKeyResponse;

pub async fn get_stream_key() -> Result<Option<String>, String> {
    let response = Request::get("/api/stream-key")
        .send()
        .await
        .map_err(|e| format!("Failed to get stream key: {}", e))?;

    let key_response: StreamKeyResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    Ok(key_response.stream_key)
}

pub async fn generate_stream_key() -> Result<StreamKeyResponse, String> {
    let response = Request::post("/api/stream-key")
        .send()
        .await
        .map_err(|e| format!("Failed to generate stream key: {}", e))?;

    response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))
}
