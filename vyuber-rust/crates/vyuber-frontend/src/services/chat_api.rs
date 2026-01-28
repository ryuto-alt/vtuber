use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use vyuber_shared::chat::ChatComment;

#[derive(Serialize)]
struct ChatRequest {
    message: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    comments: Vec<ChatComment>,
}

pub async fn send_message(message: &str) -> Result<Vec<ChatComment>, String> {
    let request_body = ChatRequest {
        message: message.to_string(),
    };

    let response = Request::post("/api/chat")
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
