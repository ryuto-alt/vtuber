use gloo_net::http::Request;
use vyuber_shared::analytics::{StreamMetadata, StreamMetadataList};

pub async fn save_metadata(metadata: StreamMetadata) -> Result<(), String> {
    Request::post("/api/analytics/save")
        .json(&serde_json::json!({ "metadata": metadata }))
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn list_metadata() -> Result<StreamMetadataList, String> {
    let resp = Request::get("/api/analytics/list")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    resp.json::<StreamMetadataList>()
        .await
        .map_err(|e| e.to_string())
}
