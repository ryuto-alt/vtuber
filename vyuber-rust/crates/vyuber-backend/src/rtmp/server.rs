use anyhow::Result;
use tracing::{info, warn};

use crate::streaming::StreamManager;

const STREAM_ID: &str = "_rtmp_default";

pub async fn start_rtmp_server(stream_manager: StreamManager) -> Result<()> {
    let sm = stream_manager.clone();

    tokio::spawn(async move {
        loop {
            // ストリームキーが設定されるまで待つ
            let _key = loop {
                if let Some(key) = sm.get_active_key().await {
                    break key;
                }
                info!("Waiting for stream key to be generated...");
                sm.wait_for_key_change().await;
            };

            let rtmp_port = sm.get_rtmp_port().await;
            info!("Starting Native RTMP listener on port {}", rtmp_port);

            // ストリーム登録
            let sender = sm.register_stream(STREAM_ID).await;

            match super::native::run_native_rtmp(
                rtmp_port,
                sender,
                sm.clone(),
                STREAM_ID.to_string(),
            ).await {
                Ok(()) => {
                    info!("RTMP session ended normally");
                }
                Err(e) => {
                    warn!("RTMP session error: {}", e);
                }
            }

            // ストリームデータリセット
            sm.remove_stream(STREAM_ID).await;
            // ポート解放を待つ（Windowsでは時間がかかる場合がある）
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            info!("Restarting RTMP listener...");
        }
    });

    Ok(())
}
