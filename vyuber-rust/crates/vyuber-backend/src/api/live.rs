use axum::{
    body::Body,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{Json, IntoResponse, Response},
};
use serde::Serialize;

use crate::streaming::StreamManager;

#[derive(Serialize)]
pub struct StreamStatusResponse {
    pub active: bool,
    pub webrtc_url: String,
    pub whep_url: String,
}

/// GET /api/live/status — MediaMTXのストリーム状態を確認
pub async fn stream_status(
    State(stream_manager): State<StreamManager>,
) -> impl IntoResponse {
    let webrtc_port = stream_manager.get_webrtc_port();
    let key = stream_manager.get_active_key().await;

    let stream_path = match &key {
        Some(k) => format!("live/{}", k),
        None => "live/stream".to_string(),
    };

    // MediaMTX APIでアクティブパスを確認
    let active = check_mediamtx_stream(&stream_path).await;

    let webrtc_url = format!("http://localhost:{}/{}", webrtc_port, stream_path);
    let whep_url = format!("http://localhost:{}/{}/whep", webrtc_port, stream_path);

    Json(StreamStatusResponse {
        active,
        webrtc_url,
        whep_url,
    })
}

/// POST /api/live/whep — MediaMTXのWHEPエンドポイントへプロキシ
pub async fn whep_proxy(
    State(stream_manager): State<StreamManager>,
    headers: HeaderMap,
    body: String,
) -> Response {
    let webrtc_port = stream_manager.get_webrtc_port();
    let key = stream_manager.get_active_key().await;
    let stream_path = match &key {
        Some(k) => format!("live/{}", k),
        None => "live/stream".to_string(),
    };

    let whep_url = format!("http://127.0.0.1:{}/{}/whep", webrtc_port, stream_path);

    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
    {
        Ok(c) => c,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    let content_type = headers
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("application/sdp");

    match client
        .post(&whep_url)
        .header("Content-Type", content_type)
        .body(body)
        .send()
        .await
    {
        Ok(resp) => {
            let status = StatusCode::from_u16(resp.status().as_u16()).unwrap_or(StatusCode::BAD_GATEWAY);
            let resp_headers = resp.headers().clone();
            let resp_body = resp.bytes().await.unwrap_or_default();

            let mut response = Response::builder().status(status);
            // Forward relevant headers
            for (name, value) in resp_headers.iter() {
                if name == "content-type" || name == "location" {
                    response = response.header(name, value);
                }
            }
            response.body(Body::from(resp_body)).unwrap_or_else(|_| StatusCode::BAD_GATEWAY.into_response())
        }
        Err(_) => StatusCode::BAD_GATEWAY.into_response(),
    }
}

/// MediaMTX APIでストリームがアクティブか確認
async fn check_mediamtx_stream(path: &str) -> bool {
    let api_url = format!(
        "http://127.0.0.1:{}/v3/paths/list",
        std::env::var("MEDIAMTX_API_PORT").unwrap_or("9997".to_string())
    );

    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(2))
        .build()
    {
        Ok(c) => c,
        Err(_) => return false,
    };

    match client.get(&api_url).send().await {
        Ok(resp) => {
            if let Ok(body) = resp.text().await {
                // パスがレスポンスに含まれていればアクティブ
                body.contains(path) && body.contains("\"ready\":true")
            } else {
                false
            }
        }
        Err(_) => false,
    }
}
