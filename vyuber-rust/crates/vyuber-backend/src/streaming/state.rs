use bytes::Bytes;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, Notify, RwLock};
use tracing::info;

const BROADCAST_CAPACITY: usize = 1024;
const HEADER_BUFFER_MAX: usize = 128 * 1024;

#[derive(Clone)]
pub struct StreamManager {
    senders: Arc<RwLock<HashMap<String, broadcast::Sender<Bytes>>>>,
    header_buffers: Arc<RwLock<HashMap<String, Vec<Bytes>>>>,
    header_sizes: Arc<RwLock<HashMap<String, usize>>>,
    has_data: Arc<RwLock<HashMap<String, bool>>>,
    /// FFmpeg再起動を通知する
    ffmpeg_restart: Arc<Notify>,
    /// データ到着を通知する（early-joiner用）
    data_ready: Arc<Notify>,
    /// 現在のストリームキー
    active_key: Arc<RwLock<Option<String>>>,
    /// RTMPポート
    rtmp_port: Arc<RwLock<u16>>,
}

impl StreamManager {
    pub fn new() -> Self {
        let rtmp_port: u16 = std::env::var("RTMP_PORT")
            .unwrap_or("1935".to_string())
            .parse()
            .unwrap_or(1935);

        Self {
            senders: Arc::new(RwLock::new(HashMap::new())),
            header_buffers: Arc::new(RwLock::new(HashMap::new())),
            header_sizes: Arc::new(RwLock::new(HashMap::new())),
            has_data: Arc::new(RwLock::new(HashMap::new())),
            ffmpeg_restart: Arc::new(Notify::new()),
            data_ready: Arc::new(Notify::new()),
            active_key: Arc::new(RwLock::new(None)),
            rtmp_port: Arc::new(RwLock::new(rtmp_port)),
        }
    }

    pub async fn register_stream(&self, key: &str) -> broadcast::Sender<Bytes> {
        let (tx, _) = broadcast::channel(BROADCAST_CAPACITY);
        let sender = tx.clone();
        self.senders.write().await.insert(key.to_string(), tx);
        self.header_buffers.write().await.insert(key.to_string(), Vec::new());
        self.header_sizes.write().await.insert(key.to_string(), 0);
        self.has_data.write().await.insert(key.to_string(), false);
        info!("Stream registered: {}", key);
        sender
    }

    pub async fn append_header(&self, key: &str, chunk: &Bytes) {
        // 初回データ到着時にhas_dataをtrueに
        {
            let mut has = self.has_data.write().await;
            if !has.get(key).copied().unwrap_or(false) {
                has.insert(key.to_string(), true);
                info!("First FLV data received for stream: {}", key);
                self.data_ready.notify_waiters();
            }
        }

        let mut sizes = self.header_sizes.write().await;
        let current = sizes.get(key).copied().unwrap_or(0);
        if current < HEADER_BUFFER_MAX {
            self.header_buffers.write().await
                .entry(key.to_string())
                .or_default()
                .push(chunk.clone());
            sizes.insert(key.to_string(), current + chunk.len());
        }
    }

    /// データが既に到着しているか（待機なし）
    pub async fn has_data_ready(&self, key: &str) -> bool {
        self.has_data.read().await.get(key).copied().unwrap_or(false)
    }

    /// データ到着を待つ（タイムアウト付き）
    pub async fn wait_for_data(&self, key: &str, timeout: std::time::Duration) -> bool {
        if self.has_data_ready(key).await {
            return true;
        }
        match tokio::time::timeout(timeout, self.data_ready.notified()).await {
            Ok(_) => self.has_data_ready(key).await,
            Err(_) => false,
        }
    }

    pub async fn get_header_chunks(&self, key: &str) -> Vec<Bytes> {
        self.header_buffers.read().await
            .get(key)
            .cloned()
            .unwrap_or_default()
    }

    pub async fn subscribe(&self, key: &str) -> Option<broadcast::Receiver<Bytes>> {
        self.senders.read().await.get(key).map(|s| s.subscribe())
    }

    /// subscribe + ヘッダー取得をアトミックに実行（データギャップ防止）
    pub async fn subscribe_with_headers(&self, key: &str) -> Option<(broadcast::Receiver<Bytes>, Vec<Bytes>)> {
        let senders = self.senders.read().await;
        let sender = senders.get(key)?;
        let receiver = sender.subscribe();
        let headers = self.header_buffers.read().await
            .get(key)
            .cloned()
            .unwrap_or_default();
        Some((receiver, headers))
    }

    pub async fn remove_stream(&self, key: &str) {
        self.senders.write().await.remove(key);
        self.header_buffers.write().await.remove(key);
        self.header_sizes.write().await.remove(key);
        self.has_data.write().await.remove(key);
        info!("Stream removed: {}", key);
    }

    /// ストリームキーを設定し、FFmpeg再起動を通知
    pub async fn set_active_key(&self, key: &str) {
        *self.active_key.write().await = Some(key.to_string());
        info!("Active stream key set: {}", key);
        self.ffmpeg_restart.notify_one();
    }

    /// 現在のストリームキーを取得
    pub async fn get_active_key(&self) -> Option<String> {
        self.active_key.read().await.clone()
    }

    /// FFmpeg再起動通知を待つ
    pub async fn wait_for_key_change(&self) {
        self.ffmpeg_restart.notified().await;
    }

    pub async fn get_rtmp_port(&self) -> u16 {
        *self.rtmp_port.read().await
    }
}
