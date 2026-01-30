use bytes::{Bytes, BytesMut};
use std::process::Stdio;
use tokio::io::AsyncReadExt;
use tokio::process::{Child, Command};
use tokio::sync::broadcast;
use tracing::{info, error};

use super::StreamManager;

/// FFmpegをRTMPリスナーモードで起動し、OBSからの配信をFLVに変換してbroadcastに流す
pub async fn start_ffmpeg_listener(
    listen_url: &str,
    sender: broadcast::Sender<Bytes>,
    stream_manager: StreamManager,
    stream_key: String,
) -> Result<Child, String> {
    info!("Starting FFmpeg RTMP listener on {}", listen_url);

    let mut cmd = Command::new("ffmpeg");
    cmd.args([
        "-fflags", "nobuffer+genpts+discardcorrupt",
        "-flags", "low_delay",
        "-probesize", "32768",
        "-analyzeduration", "0",
        "-listen", "1",
        "-i", listen_url,
        "-c", "copy",
        "-f", "flv",
        "-flvflags", "no_duration_filesize",
        "-flush_packets", "1",
        "pipe:1",
    ])
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .kill_on_drop(true);

    let mut child = cmd.spawn()
        .map_err(|e| format!("Failed to spawn FFmpeg: {}. Is FFmpeg installed?", e))?;

    let stdout = child.stdout.take().ok_or("Failed to capture FFmpeg stdout")?;
    let stderr = child.stderr.take().ok_or("Failed to capture FFmpeg stderr")?;

    // stderrログ
    tokio::spawn(async move {
        let mut reader = tokio::io::BufReader::new(stderr);
        let mut buf = vec![0u8; 4096];
        loop {
            match reader.read(&mut buf).await {
                Ok(0) => break,
                Ok(n) => {
                    let msg = String::from_utf8_lossy(&buf[..n]);
                    for line in msg.lines() {
                        if !line.is_empty() {
                            info!("[FFmpeg] {}", line);
                        }
                    }
                }
                Err(e) => {
                    error!("FFmpeg stderr read error: {}", e);
                    break;
                }
            }
        }
    });

    // stdoutからFLVデータを読み取ってbroadcast
    // 32KBバッファでシステムコール回数を削減
    let key_for_task = stream_key;
    tokio::spawn(async move {
        let mut reader = tokio::io::BufReader::with_capacity(32768, stdout);
        let mut buf = BytesMut::zeroed(32768);
        let mut total_bytes: u64 = 0;
        loop {
            match reader.read(&mut buf).await {
                Ok(0) => {
                    info!("FFmpeg stdout closed. Total bytes: {}", total_bytes);
                    break;
                }
                Ok(n) => {
                    total_bytes += n as u64;
                    // freeze()でゼロコピーのBytesに変換
                    let chunk = Bytes::copy_from_slice(&buf[..n]);

                    // ヘッダーバッファに保存（ロック1回）
                    stream_manager.append_data(&key_for_task, &chunk).await;
                    // broadcastに送信（subscriberがいない場合は無視）
                    let _ = sender.send(chunk);

                    if total_bytes <= 32768 {
                        info!("FFmpeg: received {} bytes (total: {})", n, total_bytes);
                    }
                }
                Err(e) => {
                    error!("FFmpeg stdout read error: {}", e);
                    break;
                }
            }
        }
    });

    info!("FFmpeg RTMP listener started on {}", listen_url);
    Ok(child)
}
