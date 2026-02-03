use axum::{
    extract::Multipart,
    response::Json,
};
use serde_json::{json, Value};
use crate::services::deepgram::transcribe_audio; // さっき作った機能を読み込み

// ブラウザからPOSTで呼ばれる関数
pub async fn transcribe(mut multipart: Multipart) -> Json<Value> {
    // 1. 送られてきたデータの中から「audio」という名前のファイルを探す
    while let Some(field) = multipart.next_field().await.unwrap_or(None) {
        if field.name() == Some("audio") {
            // 2. データの中身（バイト列）を取り出す
            let data = match field.bytes().await {
                Ok(bytes) => bytes,
                Err(e) => return Json(json!({ "status": "error", "message": format!("Failed to read bytes: {}", e) })),
            };

            // 3. Deepgramサービスに投げて文字起こししてもらう
            match transcribe_audio(data.to_vec()).await {
                Ok(text) => {
                    println!("🎤 音声認識成功: {}", text); // ログに出す
                    return Json(json!({ "status": "success", "text": text }));
                },
                Err(e) => {
                    eprintln!("❌ 音声認識エラー: {}", e);
                    return Json(json!({ "status": "error", "message": e }));
                }
            }
        }
    }

    // 音声ファイルが見つからなかった場合
    Json(json!({ "status": "error", "message": "No audio file found" }))
}