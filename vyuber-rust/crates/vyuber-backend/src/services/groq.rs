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

        // 会話履歴を構築（最新の発言と関連がありそうな場合のみ使用）
        let history_text = if history.is_empty() {
            String::new()
        } else {
            let history_lines: Vec<String> = history.iter()
                .enumerate()
                .map(|(i, msg)| format!("{}. 「{}」", i + 1, msg))
                .collect();
            format!("【参考：過去の発言】（最新発言と無関係なら無視してOK）\n{}\n\n", history_lines.join("\n"))
        };

        let prompt = format!(r#"YouTubeライブ配信の視聴者コメント5件を生成。

【絶対ルール】
1. 最新発言にのみ反応する。話題が変わったら過去の発言は忘れる
2. 5人全員が違う意見・反応をする（賛成、反対、質問、ボケ、共感など）
3. 同じ単語や商品名を繰り返さない
4. 日本語のみ（簡体字禁止）
5. 短いコメント（3〜12文字）

【5人の性格】
1人目(blue): 素直に反応する普通の人
2人目(green): ちょっと詳しい常連
3人目(purple): 初心者や質問する人
4人目(orange): ボケたりネタを言う人
5人目(pink): テンション高めの人

{history_text}【配信者の最新発言】「{message}」

この最新発言だけに反応。5人それぞれ違う視点で。

Output:"#, history_text = history_text, message = message);

        let request_body = GroqRequest {
            model: "llama-3.3-70b-versatile".to_string(),
            messages: vec![
                Message {
                    role: "system".to_string(),
                    content: "視聴者コメント生成AI。5人それぞれ違う性格・意見で回答。同じ言葉の繰り返し禁止。話題が変わったら過去は忘れる。JSON出力: {\"comments\":[{\"user\":\"名前\",\"text\":\"コメント\",\"color\":\"text-blue-400\"}]}".to_string(),
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
