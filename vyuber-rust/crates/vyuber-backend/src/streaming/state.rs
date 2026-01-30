use bytes::Bytes;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, Notify, RwLock};
use tracing::info;

const BROADCAST_CAPACITY: usize = 2048;
const HEADER_BUFFER_MAX: usize = 128 * 1024;

/// 1ストリームの状態（単一構造体にまとめてロック回数を最小化）
struct StreamState {
    sender: broadcast::Sender<Bytes>,
    header_chunks: Vec<Bytes>,
    header_size: usize,
    has_data: bool,
}

#[derive(Clone)]
pub struct StreamManager {
    /// ストリーム状態を単一のRwLockで管理（ロック競合を最小化）
    streams: Arc<RwLock<HashMap<String, StreamState>>>,
    /// FFmpeg再起動を通知する
    ffmpeg_restart: Arc<Notify>,
    /// データ到着を通知する（early-joiner用）
    data_ready: Arc<Notify>,
    /// 現在のストリームキー
    active_key: Arc<RwLock<Option<String>>>,
    /// RTMPポート（起動時に1回読むだけなのでAtomic）
    rtmp_port: u16,
}

impl StreamManager {
    pub fn new() -> Self {
        let rtmp_port: u16 = std::env::var("RTMP_PORT")
            .unwrap_or("1935".to_string())
            .parse()
            .unwrap_or(1935);

        Self {
            streams: Arc::new(RwLock::new(HashMap::new())),
            ffmpeg_restart: Arc::new(Notify::new()),
            data_ready: Arc::new(Notify::new()),
            active_key: Arc::new(RwLock::new(None)),
            rtmp_port,
        }
    }

    pub async fn register_stream(&self, key: &str) -> broadcast::Sender<Bytes> {
        let (tx, _) = broadcast::channel(BROADCAST_CAPACITY);
        let sender = tx.clone();
        self.streams.write().await.insert(key.to_string(), StreamState {
            sender: tx,
            header_chunks: Vec::with_capacity(32),
            header_size: 0,
            has_data: false,
        });
        info!("Stream registered: {}", key);
        sender
    }

    /// チャンクデータを追加（ホットパス — ロック1回のみ）
    pub async fn append_data(&self, key: &str, chunk: &Bytes) {
        let mut streams = self.streams.write().await;
        if let Some(state) = streams.get_mut(key) {
            if !state.has_data {
                state.has_data = true;
                info!("First FLV data received for stream: {}", key);
                self.data_ready.notify_waiters();
            }
            if state.header_size < HEADER_BUFFER_MAX {
                state.header_size += chunk.len();
                state.header_chunks.push(chunk.clone());
            }
        }
        // ロックをdropしてからbroadcast（sendはロック不要）
    }

    pub async fn has_data_ready(&self, key: &str) -> bool {
        self.streams.read().await
            .get(key)
            .map(|s| s.has_data)
            .unwrap_or(false)
    }

    pub async fn wait_for_data(&self, key: &str, timeout: std::time::Duration) -> bool {
        if self.has_data_ready(key).await {
            return true;
        }
        match tokio::time::timeout(timeout, self.data_ready.notified()).await {
            Ok(_) => self.has_data_ready(key).await,
            Err(_) => false,
        }
    }

    /// subscribe + ヘッダー取得をアトミックに実行（ロック1回）
    pub async fn subscribe_with_headers(&self, key: &str) -> Option<(broadcast::Receiver<Bytes>, Vec<Bytes>)> {
        let streams = self.streams.read().await;
        let state = streams.get(key)?;
        let receiver = state.sender.subscribe();
        let headers = state.header_chunks.clone();
        Some((receiver, headers))
    }

    pub async fn remove_stream(&self, key: &str) {
        self.streams.write().await.remove(key);
        info!("Stream removed: {}", key);
    }

    pub async fn set_active_key(&self, key: &str) {
        *self.active_key.write().await = Some(key.to_string());
        info!("Active stream key set: {}", key);
        self.ffmpeg_restart.notify_one();
    }

    pub async fn get_active_key(&self) -> Option<String> {
        self.active_key.read().await.clone()
    }

    pub async fn wait_for_key_change(&self) {
        self.ffmpeg_restart.notified().await;
    }

    pub async fn get_rtmp_port(&self) -> u16 {
        self.rtmp_port
    }
}
