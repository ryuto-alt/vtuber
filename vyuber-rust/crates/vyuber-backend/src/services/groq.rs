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

    /// 文脈付きでコメントを生成（前のコメントへのフォローアップあり）
    pub async fn generate_comments_with_context(&self, message: &str, last_comments: Option<&[ChatComment]>) -> Result<Vec<ChatComment>> {
        tracing::info!("[Chat API] Generating comments for message: {}", message);

        // 前のコメントがあれば、質問した人をピックアップ
        let context = if let Some(comments) = last_comments {
            let questions: Vec<String> = comments.iter()
                .filter(|c| c.text.contains('？') || c.text.contains('?'))
                .map(|c| format!("{}「{}」", c.user, c.text))
                .collect();
            if !questions.is_empty() {
                format!("\n【前のターンで質問した視聴者】\n{}\n→ 配信者が答えたので、この人たちは「ありがとう」「なるほど」等のリアクションをする\n", questions.join("\n"))
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        let prompt = format!(r#"YouTube配信の視聴者コメント5件を生成。
{context}
【配信者の発言】「{message}」

【コメントの種類を混ぜる】
- 共感・反応（「わかる」「それな」「草」）
- 質問（「何時まで？」「どこで？」）
- 自分の話（「俺も〇〇した」「私は〇〇派」）
- リアクション（「まじか」「えー」「おお！」）
- ボケ・ネタ（面白いツッコミ）

【ルール】
- userは日本人名（たける、ゆき、けんた、みさき等）
- textは5〜20文字くらいの自然な文
- 5人それぞれ違うタイプのコメント
- 配信者の発言をちゃんと理解して返答

【出力形式】
{{"comments":[
{{"user":"名前","text":"コメント内容","color":"text-blue-400"}},
{{"user":"名前","text":"コメント内容","color":"text-green-400"}},
{{"user":"名前","text":"コメント内容","color":"text-purple-400"}},
{{"user":"名前","text":"コメント内容","color":"text-orange-400"}},
{{"user":"名前","text":"コメント内容","color":"text-pink-400"}}
]}}"#, context = context, message = message);

        let request_body = GroqRequest {
            model: "llama-3.3-70b-versatile".to_string(),
            messages: vec![
                Message {
                    role: "system".to_string(),
                    content: "YouTube配信の視聴者コメント生成AI。自然な日本語で、5人それぞれ違うタイプのコメントを生成。質問、共感、自分の話、リアクション、ボケを混ぜる。".to_string(),
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
