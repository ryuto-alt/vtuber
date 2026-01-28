use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use vyuber_shared::chat::ChatComment;

pub struct GeminiClient {
    api_key: String,
    client: Client,
}

#[derive(Serialize)]
struct GeminiRequest {
    contents: Vec<Content>,
    #[serde(rename = "generationConfig")]
    generation_config: GenerationConfig,
}

#[derive(Serialize)]
struct Content {
    parts: Vec<Part>,
}

#[derive(Serialize)]
struct Part {
    text: String,
}

#[derive(Serialize)]
struct GenerationConfig {
    #[serde(rename = "responseMimeType")]
    response_mime_type: String,
}

#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Vec<Candidate>,
}

#[derive(Deserialize)]
struct Candidate {
    content: ResponseContent,
}

#[derive(Deserialize)]
struct ResponseContent {
    parts: Vec<ResponsePart>,
}

#[derive(Deserialize)]
struct ResponsePart {
    text: String,
}

impl GeminiClient {
    pub fn from_env() -> Self {
        let api_key = std::env::var("GEMINI_API_KEY")
            .expect("GEMINI_API_KEY must be set");

        tracing::info!("[Chat API] GEMINI_API_KEY exists: true");
        tracing::info!("[Chat API] API Key length: {}", api_key.len());

        Self {
            api_key,
            client: Client::new(),
        }
    }

    pub async fn generate_comments(&self, message: &str) -> Result<Vec<ChatComment>> {
        tracing::info!("[Chat API] Generating comments for message: {}", message);

        let prompt = format!(r#"
あなたはライブ配信の視聴者です。配信者の発言に対して、以下の5つの異なる人格になりきって、それぞれの反応を生成してください。

## 配信者の発言:
"{}"

## 生成する5つの人格:
1. 全肯定ファン (青色系): とにかく褒める。語彙力低め。
2. 初見さん (紫色系): 状況がわかっていない、または純粋な質問。
3. 辛口コメント (オレンジ色系): 少し批判的、または技術的なツッコミ。
4. スパム/ネタ勢 (ピンク色系): 絵文字多め、または文脈と関係ない勢いだけのコメント。
5. 古参 (緑色系): "おっ" "いつもの" など、慣れている感。

## 出力形式 (JSON Array):
[
  {{ "user": "名前", "text": "コメント内容", "color": "text-blue-400" }},
  ...
]

必ずValidなJSON配列のみを返してください。
"#, message);

        let request_body = GeminiRequest {
            contents: vec![Content {
                parts: vec![Part { text: prompt }],
            }],
            generation_config: GenerationConfig {
                response_mime_type: "application/json".to_string(),
            },
        };

        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/gemini-flash-latest:generateContent?key={}",
            self.api_key
        );

        tracing::info!("[Chat API] Calling Gemini API...");

        let response = self
            .client
            .post(&url)
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await?;
            tracing::error!("[Chat API] Gemini API error ({}): {}", status, error_text);
            anyhow::bail!("Gemini API returned error: {}", error_text);
        }

        let gemini_response: GeminiResponse = response.json().await?;

        let response_text = &gemini_response.candidates[0].content.parts[0].text;
        tracing::info!(
            "[Chat API] Received response: {}",
            &response_text[..response_text.len().min(100)]
        );

        // JSONとしてパース
        let comments: Vec<ChatComment> = serde_json::from_str(response_text)?;
        tracing::info!("[Chat API] Parsed JSON, comment count: {}", comments.len());

        Ok(comments)
    }
}
