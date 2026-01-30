use axum::{
    extract::State,
    response::{Response, IntoResponse},
    http::{StatusCode, header},
    body::Body,
};
use bytes::Bytes;
use futures_util::stream::StreamExt;
use tokio_stream::wrappers::BroadcastStream;

use crate::streaming::StreamManager;

const STREAM_ID: &str = "_rtmp_default";

/// HTTP-FLVストリーミングエンドポイント
///
/// GET /api/live/stream
pub async fn stream_flv(
    State(stream_manager): State<StreamManager>,
) -> Response {
    tracing::info!("FLV stream request received");

    // データ到着を待つ（最大30秒）
    // subscribe前にwaitすることで、subscribe→データ取得の間のギャップを最小化
    let data_ready = stream_manager.wait_for_data(STREAM_ID, std::time::Duration::from_secs(30)).await;
    if !data_ready {
        tracing::info!("Stream not ready, returning 503");
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            "Stream not ready",
        ).into_response();
    }

    // データ到着済み → subscribe + ヘッダー取得をアトミックに実行
    let (receiver, header_chunks) = match stream_manager.subscribe_with_headers(STREAM_ID).await {
        Some(result) => result,
        None => {
            tracing::warn!("Stream disappeared for {}", STREAM_ID);
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                "Stream ended",
            ).into_response();
        }
    };

    tracing::info!("Streaming FLV: {} header chunks + live data", header_chunks.len());

    let header_stream = futures_util::stream::iter(
        header_chunks.into_iter().map(Ok::<Bytes, std::io::Error>)
    );

    let live_stream = BroadcastStream::new(receiver)
        .filter_map(|result: Result<Bytes, _>| {
            futures_util::future::ready(result.ok())
        })
        .map(Ok::<Bytes, std::io::Error>);

    let body = Body::from_stream(header_stream.chain(live_stream));

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "video/x-flv")
        .header(header::CACHE_CONTROL, "no-cache, no-store")
        .header(header::CONNECTION, "keep-alive")
        .header(header::TRANSFER_ENCODING, "chunked")
        .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
        .body(body)
        .unwrap()
}
