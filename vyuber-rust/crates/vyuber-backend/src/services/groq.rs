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
    pub async fn generate_comments_with_history(&self, message: &str, _history: &[String]) -> Result<Vec<ChatComment>> {
        tracing::info!("[Chat API] Generating comments for message: {}", message);

        let prompt = format!(r#"配信者「{message}」に対する視聴者コメント5件をJSON生成。

【ルール】
- userは日本人の名前（例：たける、ゆき、けんた）
- textは配信者への自然な返答（3〜10文字）
- 5人それぞれ違う反応をする
- 配信者の発言の意味を理解して答える

【例1】配信者「電車遅延えぐかった」
{{"comments":[
{{"user":"たける","text":"まじか","color":"text-blue-400"}},
{{"user":"ゆうき","text":"何分遅れた？","color":"text-green-400"}},
{{"user":"みさき","text":"大変だったね","color":"text-purple-400"}},
{{"user":"けんた","text":"俺も遅刻した","color":"text-orange-400"}},
{{"user":"あやか","text":"おつかれ！","color":"text-pink-400"}}
]}}

【例2】配信者「お腹すいた」
{{"comments":[
{{"user":"そうた","text":"俺も","color":"text-blue-400"}},
{{"user":"りな","text":"何食べる？","color":"text-green-400"}},
{{"user":"たくみ","text":"ラーメン行こ","color":"text-purple-400"}},
{{"user":"ゆい","text":"カップ麺ある？","color":"text-orange-400"}},
{{"user":"はると","text":"食べよ食べよ！","color":"text-pink-400"}}
]}}

【例3】配信者「〇〇ってやばくね？w」
{{"comments":[
{{"user":"けい","text":"草","color":"text-blue-400"}},
{{"user":"まさき","text":"それなw","color":"text-green-400"}},
{{"user":"あおい","text":"わかるw","color":"text-purple-400"}},
{{"user":"りく","text":"まじでやばい","color":"text-orange-400"}},
{{"user":"ゆな","text":"wwwww","color":"text-pink-400"}}
]}}

【例4】配信者「下ネタや暴言」
{{"comments":[
{{"user":"たろう","text":"おいw","color":"text-blue-400"}},
{{"user":"けんじ","text":"草","color":"text-green-400"}},
{{"user":"みく","text":"やめろw","color":"text-purple-400"}},
{{"user":"そうた","text":"配信終わるぞ","color":"text-orange-400"}},
{{"user":"りさ","text":"BANされるw","color":"text-pink-400"}}
]}}

配信者「{message}」"#, message = message);

        let request_body = GroqRequest {
            model: "llama-3.3-70b-versatile".to_string(),
            messages: vec![
                Message {
                    role: "system".to_string(),
                    content: "YouTube配信の視聴者コメントをJSON生成。userは日本人名（たける、ゆき等）、textは短い自然な返答。".to_string(),
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
