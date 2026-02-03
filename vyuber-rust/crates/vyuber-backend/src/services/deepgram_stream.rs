use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::Response,
};
use futures::{sink::SinkExt, stream::StreamExt};
use std::env;
use tokio_tungstenite::{
    connect_async, 
    tungstenite::{
        protocol::Message as TungsteniteMessage,
        handshake::client::generate_key
    }
};
use url::Url;
use http::Request;

pub async fn handler(ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut client_socket: WebSocket) {
    let api_key = env::var("DEEPGRAM_API_KEY").unwrap_or_default();

    // Deepgram URL (日本語, WebM自動認識)
    let url = Url::parse("wss://api.deepgram.com/v1/listen?language=ja&smart_format=true&interim_results=true&model=nova-2").unwrap();
    
    let request = Request::builder()
        .uri(url.as_str())
        .header("Authorization", format!("Token {}", api_key))
        .header("Sec-WebSocket-Key", generate_key())
        .header("Sec-WebSocket-Version", "13")
        .header("Connection", "Upgrade")
        .header("Upgrade", "websocket")
        .header("Host", url.host_str().unwrap_or("api.deepgram.com"))
        .body(())
        .unwrap();

    let (mut deepgram_socket, _) = match connect_async(request).await {
        Ok(s) => s,
        Err(e) => {
            println!("❌ Deepgram connection failed: {}", e);
            return;
        }
    };

    println!("🔴 リアルタイム音声認識を開始");

    let (mut client_sender, mut client_receiver) = client_socket.split();
    let (mut deepgram_sender, mut deepgram_receiver) = deepgram_socket.split();

    // タスクA: ブラウザ → Deepgram
    let mut send_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = client_receiver.next().await {
            if let Message::Binary(data) = msg {
                // ★ログ有効化: ここでデータが来ているかチェック！
                println!("🎤 音声受信: {} bytes -> Deepgramへ転送", data.len());
                
                if deepgram_sender.send(TungsteniteMessage::Binary(data)).await.is_err() {
                    println!("⚠️ Deepgramへの送信エラー");
                    break;
                }
            } else if let Message::Close(_) = msg {
                println!("ブラウザが接続を切断しました");
                break;
            }
        }
    });

    // タスクB: Deepgram → ブラウザ
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = deepgram_receiver.next().await {
            if let TungsteniteMessage::Text(text) = msg {
                // ★ログ有効化: 返信内容をすべて表示
                println!("🤖 Deepgram返信: {}", text);
                
                if client_sender.send(Message::Text(text)).await.is_err() {
                    println!("⚠️ ブラウザへの送信エラー");
                    break;
                }
            }
        }
    });

    tokio::select! {
        _ = (&mut send_task) => {
            println!("send_task finished");
            recv_task.abort();
        },
        _ = (&mut recv_task) => {
            println!("recv_task finished");
            send_task.abort();
        },
    };
    
    println!("⚪ リアルタイム音声認識を終了");
}