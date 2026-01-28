use axum::{
    extract::Path,
    response::{Response, IntoResponse},
    http::{StatusCode, header},
};

/// HTTP-FLVストリーミングエンドポイント
///
/// MVP版では実装をスキップ
/// 実際の実装にはFFmpegからのFLVストリームを
/// HTTPレスポンスとしてストリーミングする必要がある
pub async fn stream_flv(
    Path(stream_key): Path<String>,
) -> Response {
    tracing::info!("FLV stream request for key: {}", stream_key);

    // MVP版: プレースホルダーレスポンス
    // 実装時はFFmpegプロセスの出力をストリーミング

    (
        StatusCode::NOT_IMPLEMENTED,
        [(header::CONTENT_TYPE, "text/plain")],
        "HTTP-FLV streaming not yet implemented. \
         This will stream video from FFmpeg transcoder in production."
    ).into_response()
}
