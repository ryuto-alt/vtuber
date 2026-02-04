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

    pub async fn generate_comments(&self, message: &str) -> Result<Vec<ChatComment>> {
        tracing::info!("[Chat API] Generating comments for message: {}", message);

        // Few-shot examples for high accuracy responses
        let prompt = format!(r#"YouTubeライブ配信の視聴者コメントを5件生成。

【絶対ルール】
- 日本語（ひらがな・カタカナ・漢字）のみ使用。簡体字・繁体字は禁止
- 配信者が質問したら必ず質問に答える（はい/いいえ、する/しない等）
- 短いコメント（1〜12文字）

【Examples】

Input: 「こんばんはー！」
Output: {{"comments":[
{{"user":"たける","text":"こんばんは！","color":"text-blue-400"}},
{{"user":"ゆき","text":"待ってた！","color":"text-green-400"}},
{{"user":"初見","text":"初見です！","color":"text-purple-400"}},
{{"user":"ねこまる","text":"ばんちゃ！","color":"text-orange-400"}},
{{"user":"さくら","text":"きたー！","color":"text-pink-400"}}
]}}

Input: 「みんな元気してた？」
Output: {{"comments":[
{{"user":"けんた","text":"元気だよ！","color":"text-blue-400"}},
{{"user":"まりこ","text":"ぼちぼち","color":"text-green-400"}},
{{"user":"たろう","text":"絶好調！","color":"text-purple-400"}},
{{"user":"しょうた","text":"まあまあ","color":"text-orange-400"}},
{{"user":"みく","text":"元気！","color":"text-pink-400"}}
]}}

Input: 「起業しないの？」
Output: {{"comments":[
{{"user":"けんじ","text":"しないかな","color":"text-blue-400"}},
{{"user":"あきら","text":"興味ある！","color":"text-green-400"}},
{{"user":"社会人","text":"いつかしたい","color":"text-purple-400"}},
{{"user":"たくや","text":"リスク怖い","color":"text-orange-400"}},
{{"user":"ゆうき","text":"するつもり！","color":"text-pink-400"}}
]}}

Input: 「ゲームやる人いる？」
Output: {{"comments":[
{{"user":"ゲーマー","text":"やるよ！","color":"text-blue-400"}},
{{"user":"かずき","text":"毎日やる","color":"text-green-400"}},
{{"user":"みさき","text":"たまにやる","color":"text-purple-400"}},
{{"user":"りょう","text":"最近やってない","color":"text-orange-400"}},
{{"user":"はると","text":"やりたい！","color":"text-pink-400"}}
]}}

Input: 「彼女いる人？」
Output: {{"comments":[
{{"user":"たけし","text":"いるよ！","color":"text-blue-400"}},
{{"user":"ぼっち","text":"いない…","color":"text-green-400"}},
{{"user":"りく","text":"募集中","color":"text-purple-400"}},
{{"user":"かい","text":"いません","color":"text-orange-400"}},
{{"user":"そうた","text":"秘密","color":"text-pink-400"}}
]}}

Input: 「今日カレー食べた」
Output: {{"comments":[
{{"user":"りょう","text":"いいな","color":"text-blue-400"}},
{{"user":"あや","text":"何カレー？","color":"text-green-400"}},
{{"user":"ともや","text":"食べたい","color":"text-purple-400"}},
{{"user":"けい","text":"手作り？","color":"text-orange-400"}},
{{"user":"なな","text":"辛口？","color":"text-pink-400"}}
]}}

Input: 「{0}」
Output:"#, message);

        let request_body = GroqRequest {
            model: "llama-3.3-70b-versatile".to_string(),
            messages: vec![
                Message {
                    role: "system".to_string(),
                    content: "JSON生成AI。日本語のみ使用（簡体字禁止）。質問には必ず答える形で回答。例に従いJSONのみ出力。".to_string(),
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
