use anyhow::Result;
use tracing::{info, warn};

use crate::streaming::StreamManager;

const STREAM_ID: &str = "_rtmp_default";

pub async fn start_rtmp_server(stream_manager: StreamManager) -> Result<()> {
    let sm = stream_manager.clone();

    tokio::spawn(async move {
        loop {
            // ストリームキーが設定されるまで待つ
            let key = loop {
                if let Some(key) = sm.get_active_key().await {
                    break key;
                }
                info!("Waiting for stream key to be generated...");
                sm.wait_for_key_change().await;
            };

            let rtmp_port = sm.get_rtmp_port().await;
            let listen_url = format!("rtmp://0.0.0.0:{}/live/{}", rtmp_port, key);
            info!("Starting FFmpeg RTMP listener: {}", listen_url);

            // ストリーム登録
            let sender = sm.register_stream(STREAM_ID).await;

            match crate::streaming::ffmpeg::start_ffmpeg_listener(
                &listen_url,
                sender,
                sm.clone(),
                STREAM_ID.to_string(),
            ).await {
                Ok(mut child) => {
                    info!("FFmpeg RTMP listener active on {}", listen_url);

                    // FFmpeg終了 OR キー変更を待つ
                    tokio::select! {
                        status = child.wait() => {
                            match status {
                                Ok(s) => warn!("FFmpeg exited: {}. Will restart when OBS reconnects...", s),
                                Err(e) => warn!("FFmpeg wait error: {}", e),
                            }
                        }
                        _ = sm.wait_for_key_change() => {
                            info!("Stream key changed, killing current FFmpeg...");
                            let _ = child.kill().await;
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to start FFmpeg: {}", e);
                }
            }

            // ストリームデータリセット
            sm.remove_stream(STREAM_ID).await;
            // ポート解放を待つ（Windowsでは時間がかかる場合がある）
            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
            info!("Restarting FFmpeg RTMP listener...");
        }
    });

    Ok(())
}
