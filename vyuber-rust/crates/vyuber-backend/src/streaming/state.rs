use std::sync::Arc;
use tokio::sync::{Notify, RwLock};
use tracing::info;

/// MediaMTX統合版 StreamManager
/// 自前のRTMP/FLV処理は廃止し、MediaMTXプロセス管理 + ストリームキー管理のみ
#[derive(Clone)]
pub struct StreamManager {
    /// MediaMTX再起動を通知する
    restart_notify: Arc<Notify>,
    /// 現在のストリームキー
    active_key: Arc<RwLock<Option<String>>>,
    /// RTMPポート（MediaMTXが使用）
    rtmp_port: u16,
    /// WebRTCポート（MediaMTXが使用）
    webrtc_port: u16,
}

impl StreamManager {
    pub fn new() -> Self {
        let rtmp_port: u16 = std::env::var("RTMP_PORT")
            .unwrap_or("1935".to_string())
            .parse()
            .unwrap_or(1935);

        let webrtc_port: u16 = std::env::var("WEBRTC_PORT")
            .unwrap_or("8889".to_string())
            .parse()
            .unwrap_or(8889);

        Self {
            restart_notify: Arc::new(Notify::new()),
            active_key: Arc::new(RwLock::new(None)),
            rtmp_port,
            webrtc_port,
        }
    }

    pub async fn set_active_key(&self, key: &str) {
        *self.active_key.write().await = Some(key.to_string());
        info!("Active stream key set: {}", key);
        self.restart_notify.notify_one();
    }

    pub async fn get_active_key(&self) -> Option<String> {
        self.active_key.read().await.clone()
    }

    pub async fn wait_for_key_change(&self) {
        self.restart_notify.notified().await;
    }

    pub fn get_rtmp_port(&self) -> u16 {
        self.rtmp_port
    }

    pub fn get_webrtc_port(&self) -> u16 {
        self.webrtc_port
    }
}
