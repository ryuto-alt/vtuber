use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use vyuber_shared::chat::{ChatComment, ChatRequest, ChatMode};

#[derive(Deserialize)]
struct ChatResponse {
    comments: Vec<ChatComment>,
}

// mode引数を追加
pub async fn send_message(message: &str, mode: ChatMode) -> Result<Vec<ChatComment>, String> {
    let request_body = ChatRequest {
        message: message.to_string(),
        mode,
    };

    let response = Request::post("http://127.0.0.1:3000/api/chat")
        .json(&request_body)
        .map_err(|e| format!("Failed to serialize request: {}", e))?
        .send()
        .await
        .map_err(|e| format!("Failed to send request: {}", e))?;

    if !response.ok() {
        return Err(format!("API error: {}", response.status()));
    }

    let chat_response: ChatResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    Ok(chat_response.comments)
}