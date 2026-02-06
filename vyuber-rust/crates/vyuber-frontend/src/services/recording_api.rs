use gloo_net::http::Request;
use serde::Serialize;

#[derive(Serialize)]
struct RecordingRequest {
    action: String,
    stream_key: String,
}

pub async fn start_recording(stream_key: &str) -> Result<(), String> {
    let req = RecordingRequest {
        action: "start".to_string(),
        stream_key: stream_key.to_string(),
    };

    Request::post("/api/recording")
        .json(&req)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn stop_recording(stream_key: &str) -> Result<(), String> {
    let req = RecordingRequest {
        action: "stop".to_string(),
        stream_key: stream_key.to_string(),
    };

    Request::post("/api/recording")
        .json(&req)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}
