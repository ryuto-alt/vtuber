use anyhow::Result;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio::io::AsyncReadExt;
use tracing::{info, error};

/// RTMPサーバーを起動
///
/// MVP版: プレースホルダー実装
/// 実際のRTMP実装にはsheave等のライブラリが必要
pub async fn start_rtmp_server() -> Result<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 1935));

    info!("RTMP server (placeholder) will listen on {}", addr);
    info!("Full RTMP implementation pending - sheave integration required");

    // バックグラウンドでリスナーを起動
    tokio::spawn(async move {
        match TcpListener::bind(addr).await {
            Ok(listener) => {
                info!("RTMP placeholder server listening on {}", addr);

                loop {
                    match listener.accept().await {
                        Ok((mut socket, peer_addr)) => {
                            info!("New RTMP connection from: {}", peer_addr);

                            tokio::spawn(async move {
                                let mut buf = vec![0; 1024];

                                // データを受信してログに記録
                                loop {
                                    match socket.read(&mut buf).await {
                                        Ok(0) => {
                                            info!("RTMP connection closed: {}", peer_addr);
                                            break;
                                        }
                                        Ok(n) => {
                                            info!("Received {} bytes from RTMP client", n);
                                            // MVP: データは処理せず、ログのみ
                                        }
                                        Err(e) => {
                                            error!("RTMP read error: {}", e);
                                            break;
                                        }
                                    }
                                }
                            });
                        }
                        Err(e) => {
                            error!("Failed to accept RTMP connection: {}", e);
                        }
                    }
                }
            }
            Err(e) => {
                error!("Failed to bind RTMP server: {}", e);
            }
        }
    });

    Ok(())
}
