use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use vyuber_shared::chat::ChatComment;

pub struct GroqClient {
    api_key: String,
    client: Client,
}

#[derive(Serialize)]
struct GroqRequest {
    model: String,
    messages: Vec<Message>,
    response_format: ResponseFormat,
    max_tokens: u32,
}

#[derive(Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct ResponseFormat {
    #[serde(rename = "type")]
    format_type: String,
}

#[derive(Deserialize)]
struct GroqResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: ResponseMessage,
}

#[derive(Deserialize)]
struct ResponseMessage {
    content: String,
}

#[derive(Deserialize)]
struct CommentsWrapper {
    comments: Vec<ChatComment>,
}

impl GroqClient {
    pub fn from_env() -> Self {
        let api_key = std::env::var("GROQ_API_KEY")
            .expect("GROQ_API_KEY must be set");

        tracing::info!("[Chat API] GROQ_API_KEY exists: true");
        tracing::info!("[Chat API] API Key length: {}", api_key.len());

        Self {
            api_key,
            client: Client::new(),
        }
    }

    /// 会話履歴付きでコメントを生成
    pub async fn generate_comments_with_history(&self, message: &str, history: &[String]) -> Result<Vec<ChatComment>> {
        tracing::info!("[Chat API] Generating comments for message: {}", message);
        tracing::info!("[Chat API] Using {} history messages", history.len());

        // 会話履歴を構築
        let history_text = if history.is_empty() {
            String::new()
        } else {
            let history_lines: Vec<String> = history.iter()
                .enumerate()
                .map(|(i, msg)| format!("{}. 「{}」", i + 1, msg))
                .collect();
            format!("【配信者の過去の発言】\n{}\n\n", history_lines.join("\n"))
        };

        let prompt = format!(r#"YouTubeライブ配信の視聴者コメントを5件生成。

【絶対ルール】
- 日本語（ひらがな・カタカナ・漢字）のみ使用。簡体字禁止
- 配信者の質問には必ず答える
- 過去の発言を踏まえて文脈に合った返答をする
- 金額を聞かれたら具体的な金額で答える
- 短いコメント（1〜15文字）

{history_text}【配信者の最新発言】「{message}」

上記の最新発言に対するコメントを生成。過去の発言があれば文脈を考慮すること。

Output:"#, history_text = history_text, message = message);

        let request_body = GroqRequest {
            model: "llama-3.3-70b-versatile".to_string(),
            messages: vec![
                Message {
                    role: "system".to_string(),
                    content: "YouTube配信の視聴者コメント生成AI。日本語のみ。配信者の過去の発言を踏まえて文脈に合った返答をする。金額の質問には「10万円」「20万」など具体的に。JSON形式で出力: {\"comments\":[{\"user\":\"名前\",\"text\":\"コメント\",\"color\":\"text-blue-400\"}]}".to_string(),
                },
                Message {
                    role: "user".to_string(),
                    content: prompt,
                },
            ],
            response_format: ResponseFormat {
                format_type: "json_object".to_string(),
            },
            max_tokens: 1024,
        };

        tracing::info!("[Chat API] Calling Groq API...");

        let response = self
            .client
            .post("https://api.groq.com/openai/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await?;
            tracing::error!("[Chat API] Groq API error ({}): {}", status, error_text);
            anyhow::bail!("Groq API returned error: {}", error_text);
        }

        let groq_response: GroqResponse = response.json().await?;

        let response_text = &groq_response.choices[0].message.content;
        let preview: String = response_text.chars().take(100).collect();
        tracing::info!("[Chat API] Received response: {}", preview);

        // JSONとしてパース
        let wrapper: CommentsWrapper = serde_json::from_str(response_text)?;
        tracing::info!("[Chat API] Parsed JSON, comment count: {}", wrapper.comments.len());

        Ok(wrapper.comments)
    }
}
