use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use vyuber_shared::chat::ChatComment;
use crate::api::personas::Persona;

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
struct RawComment {
    user: String,
    text: String,
}

#[derive(Deserialize)]
struct CommentsWrapper {
    comments: Vec<RawComment>,
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

    pub async fn generate_comments_with_context(
        &self, 
        message: &str, 
        last_comments: Option<&[ChatComment]>,
        active_personas: &[Persona]
    ) -> Result<Vec<ChatComment>> {
        tracing::info!("[Chat API] Generating comments for message: {}", message);

        // ★ここがエラーの原因でした！新しい項目に合わせて修正済みです
        let personas_prompt = active_personas.iter().map(|p| {
            format!(
                "- 名前: {}\n  属性: {} / {} / {}\n  関係: {}\n  興味: {}\n  性格・口調: {}", 
                p.name, p.age, p.job, p.location, p.relationship, p.interest, p.tone
            )
        }).collect::<Vec<_>>().join("\n\n");

        let context = if let Some(comments) = last_comments {
            let users: Vec<String> = comments.iter()
                .map(|c| format!("{}「{}」", c.user, c.text))
                .collect();
            format!("【直前のチャット履歴】\n{}", users.join("\n"))
        } else {
            "【直前のチャット履歴】\n(なし)".to_string()
        };

        // システムプロンプト
        let system_prompt = format!(r#"
あなたはYouTube配信のチャット欄を盛り上げる「視聴者シミュレーター」です。
今回は、以下の【選抜された視聴者】になりきってコメントを生成してください。
各視聴者の「年齢」「職業」「関係性」などの背景情報を踏まえ、リアルな発言を心がけてください。

【今回の選抜視聴者リスト（詳細プロフィール）】
{personas_prompt}

【生成ルール】
1. 出力は必ずJSON形式 (comments配列) にすること。
2. 上記リストにいる全員分のコメントを1つずつ生成すること。
3. "user" はリストの名前をそのまま使うこと。
4. "text" はそのキャラの性格・口調を完全に再現すること。
   - 興味のない話題には適当に反応したり、自分の興味のある話題に無理やり繋げてもよい。
   - アンチや指示厨は、少し棘のある言い方をすること。
5. "color" はJSONには含めなくてよい。
6. 文脈を読み、配信者の発言に対して自然な反応をすること。

【出力例】
{{
  "comments": [
    {{ "user": "草野", "text": "噛んだｗｗｗ" }},
    {{ "user": "博士", "text": "今の現象はラグではなく仕様ですね" }}
  ]
}}
"#);

        let user_message = format!("【配信者の発言】\n「{}」\n\n{}", message, context);

        let request_body = GroqRequest {
            model: "llama-3.3-70b-versatile".to_string(),
            messages: vec![
                Message { role: "system".to_string(), content: system_prompt },
                Message { role: "user".to_string(), content: user_message },
            ],
            response_format: ResponseFormat { format_type: "json_object".to_string() },
            max_tokens: 1024,
        };

        tracing::info!("[Chat API] Calling Groq API...");

        let response = self.client
            .post("https://api.groq.com/openai/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
             let status = response.status();
             let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
             tracing::error!("[Chat API] Groq API error ({}): {}", status, error_text);
             anyhow::bail!("Groq API returned error: {}", error_text);
        }

        let response_text_raw = response.text().await?;
        tracing::info!("[Chat API] Raw response: {}", response_text_raw);

        let groq_response: GroqResponse = serde_json::from_str(&response_text_raw)
            .map_err(|e| anyhow::anyhow!("Failed to parse GroqResponse: {}", e))?;
        
        let content_json_str = &groq_response.choices[0].message.content;
        
        let wrapper: CommentsWrapper = serde_json::from_str(content_json_str)
            .map_err(|e| anyhow::anyhow!("Failed to parse content JSON: {}", e))?;

        // 色情報の復元
        let mut final_comments = Vec::new();
        for raw in wrapper.comments {
            let color = active_personas.iter()
                .find(|p| p.name == raw.user)
                .map(|p| p.color.clone())
                .unwrap_or_else(|| "text-slate-400".to_string());

            final_comments.push(ChatComment {
                user: raw.user,
                text: raw.text,
                color,
            });
        }

        tracing::info!("[Chat API] Successfully generated {} comments", final_comments.len());
        Ok(final_comments)
    }
}