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
        let prompt = format!(r#"あなたはYouTubeライブ配信の視聴者コメントを生成するAIです。
配信者の発言に対して、自然で関連性のある視聴者コメントを5件生成してください。

【ルール】
1. 配信者の発言内容に必ず関連したコメントを生成する
2. 質問には答える、報告には反応する、挨拶には挨拶を返す
3. 短く自然なコメント（1〜15文字程度）
4. バリエーション豊かに（同じような返答を避ける）

【Few-shot Examples】

Input: 「こんばんはー！」
Output: {{"comments":[
{{"user":"たける","text":"こんばんは！","color":"text-blue-400"}},
{{"user":"ゆき@配信好き","text":"待ってた！","color":"text-green-400"}},
{{"user":"初見です","text":"初見です！","color":"text-purple-400"}},
{{"user":"ねこまる","text":"ばんちゃ！","color":"text-orange-400"}},
{{"user":"さくら","text":"今日も来たよ〜","color":"text-pink-400"}}
]}}

Input: 「みんな元気してた？」
Output: {{"comments":[
{{"user":"けんた","text":"元気だよ！","color":"text-blue-400"}},
{{"user":"まりこ","text":"ぼちぼちかな","color":"text-green-400"}},
{{"user":"ゲーマー太郎","text":"絶好調！","color":"text-purple-400"}},
{{"user":"しょうた","text":"まあまあ〜","color":"text-orange-400"}},
{{"user":"みく","text":"元気元気！そっちは？","color":"text-pink-400"}}
]}}

Input: 「今日カレー食べたんだよね」
Output: {{"comments":[
{{"user":"りょう","text":"いいな〜","color":"text-blue-400"}},
{{"user":"カレー好き","text":"何カレー？","color":"text-green-400"}},
{{"user":"ともや","text":"俺も食べたい","color":"text-purple-400"}},
{{"user":"あやか","text":"手作り？","color":"text-orange-400"}},
{{"user":"たくみ","text":"辛口派？甘口派？","color":"text-pink-400"}}
]}}

Input: 「今日仕事疲れた〜」
Output: {{"comments":[
{{"user":"しんじ","text":"おつかれ！","color":"text-blue-400"}},
{{"user":"OL子","text":"わかる…","color":"text-green-400"}},
{{"user":"だいき","text":"ゆっくり休んで","color":"text-purple-400"}},
{{"user":"はるな","text":"配信で癒されて","color":"text-orange-400"}},
{{"user":"こうへい","text":"何があったの？","color":"text-pink-400"}}
]}}

Input: 「今日はマイクラやるよ！」
Output: {{"comments":[
{{"user":"マイクラ勢","text":"きたー！","color":"text-blue-400"}},
{{"user":"ゆうと","text":"待ってました！","color":"text-green-400"}},
{{"user":"建築好き","text":"何作るの？","color":"text-purple-400"}},
{{"user":"れん","text":"サバイバル？","color":"text-orange-400"}},
{{"user":"あおい","text":"楽しみ！","color":"text-pink-400"}}
]}}

Input: 「最近寝不足なんだよね」
Output: {{"comments":[
{{"user":"けい","text":"大丈夫？","color":"text-blue-400"}},
{{"user":"夜更かし民","text":"わかりみ","color":"text-green-400"}},
{{"user":"みさき","text":"ちゃんと寝て！","color":"text-purple-400"}},
{{"user":"そうた","text":"何時に寝てる？","color":"text-orange-400"}},
{{"user":"なつみ","text":"体調気をつけてね","color":"text-pink-400"}}
]}}

---
Now generate comments for this input:

Input: 「{0}」
Output:"#, message);

        let request_body = GroqRequest {
            model: "llama-3.3-70b-versatile".to_string(),
            messages: vec![
                Message {
                    role: "system".to_string(),
                    content: "あなたはJSON生成専用AIです。例に従って、配信者の発言に関連したコメントのみを生成してください。無関係な定型文は禁止。Output:の後にJSONのみ出力。".to_string(),
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
