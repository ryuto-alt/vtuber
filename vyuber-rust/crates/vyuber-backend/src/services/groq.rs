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

        // 前のコメントがあれば、視聴者情報を構築
        let context = if let Some(comments) = last_comments {
            let mut context_parts = Vec::new();

            // 前のターンの視聴者リスト
            let users: Vec<String> = comments.iter()
                .map(|c| format!("{}「{}」", c.user, c.text))
                .collect();
            context_parts.push(format!("【前のターンの視聴者コメント】\n{}", users.join("\n")));

            // 配信者が誰かの名前を呼んでいるかチェック
            let mentioned_users: Vec<&ChatComment> = comments.iter()
                .filter(|c| message.contains(&c.user))
                .collect();

            if !mentioned_users.is_empty() {
                let names: Vec<String> = mentioned_users.iter()
                    .map(|c| c.user.clone())
                    .collect();
                context_parts.push(format!(
                    "\n【重要】配信者が「{}」に話しかけています。この人は必ず返答してください。1人目のコメントにしてください。",
                    names.join("、")
                ));
            }

            // 質問した人
            let questions: Vec<String> = comments.iter()
                .filter(|c| c.text.contains('？') || c.text.contains('?'))
                .map(|c| c.user.clone())
                .collect();
            if !questions.is_empty() && mentioned_users.is_empty() {
                context_parts.push(format!(
                    "\n【質問した視聴者】{}\n→ 配信者が答えたので、この人たちは「ありがとう」「なるほど」等のリアクションをする可能性あり",
                    questions.join("、")
                ));
            }

            context_parts.join("\n")
        } else {
            String::new()
        };

        let prompt = format!(r#"YouTube配信の視聴者コメント5件を生成。
{context}
【配信者の発言】「{message}」

【5人の役割（必ずこの通りに）】
1. 短いリアクション（3〜8文字）「草」「まじか」「それな」など
2. 質問する人（10〜20文字）「〇〇ってどうなの？」「何で〇〇したの？」
3. 自分の体験を語る人（20〜40文字）「俺も昔〇〇したことあるけど〜」「私の場合は〜だったな」
4. 長文で熱く語る人（30〜50文字）体験談や意見を詳しく書く
5. ボケ・ネタ担当（5〜15文字）面白いツッコミやボケ

【ルール】
- userは日本人名（たける、ゆき、けんた、みさき、りょう等）
- 配信者の発言をちゃんと理解して、その話題に沿った返答をする
- 長文の人は本当に長く書く（短くしない）
- 配信者が誰かの名前を呼んでいたら、その人が必ず1人目で返答する
- 前のターンにいた視聴者は同じ名前で再登場してもOK

【出力例】配信者「彼女彼氏おる人どんくらいいる？」
{{"comments":[
{{"user":"たける","text":"いるよ","color":"text-blue-400"}},
{{"user":"ゆき","text":"みんなどうやって出会ったの？","color":"text-green-400"}},
{{"user":"けんた","text":"俺は去年マッチングアプリで出会って付き合い始めた","color":"text-purple-400"}},
{{"user":"まさき","text":"自分は高校の時から付き合ってる彼女いるけど、最近ちょっと倦怠期かもしれん...みんなどう乗り越えてる？","color":"text-orange-400"}},
{{"user":"りな","text":"ぼっちです泣","color":"text-pink-400"}}
]}}

【配信者の発言】「{message}」"#, context = context, message = message);

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
