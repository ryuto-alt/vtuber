use reqwest::Client;
use serde_json::Value;
use std::env;

// 音声データ(バイナリ)を受け取って、文字(String)を返す関数
pub async fn transcribe_audio(audio_data: Vec<u8>) -> Result<String, String> {
    // 1. InfisicalからAPIキーを取得
    let api_key = env::var("DEEPGRAM_API_KEY")
        .map_err(|_| "Deepgram API key not found in environment variables".to_string())?;

    // 2. Deepgram Nova-2 に送信
    let client = Client::new();
    let response = client
        .post("https://api.deepgram.com/v1/listen?model=nova-2&smart_format=true&language=ja")
        .header("Authorization", format!("Token {}", api_key))
        .header("Content-Type", "audio/*")
        .body(audio_data)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    // 3. エラーチェック（ここを修正しました）
    if !response.status().is_success() {
        let status = response.status(); // 先にステータスを保存
        let error_text = response.text().await.unwrap_or_default(); // その後で中身を読む
        return Err(format!("Deepgram API Error: {} - {}", status, error_text));
    }

    // 4. 結果（JSON）から文字だけ抜き出す
    let json: Value = response.json().await.map_err(|e| format!("Parse error: {}", e))?;
    
    let transcript = json["results"]["channels"][0]["alternatives"][0]["transcript"]
        .as_str()
        .unwrap_or("")
        .to_string();

    Ok(transcript)
}