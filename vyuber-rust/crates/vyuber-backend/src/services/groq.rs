use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use vyuber_shared::chat::ChatComment;
use vyuber_shared::agent::{Persona, AgentTier};

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
        active_personas: &[Persona],
        model_id: &str,
        system_instruction: &str,
    ) -> Result<Vec<ChatComment>> {
        
        // ★修正: Tierに応じたプロンプト構築
        let personas_prompt = active_personas.iter().map(|p| {
            match p.tier {
                AgentTier::Anchor => format!(
                    "- 名前: {}\n  属性/詳細: {}\n  口調: {}", 
                    p.name, p.bio, p.tone
                ),
                AgentTier::Swarm => format!(
                    "- 名前: {}\n  反応パターン: {}", 
                    p.name, p.tone
                ),
            }
        }).collect::<Vec<_>>().join("\n\n");

        let context = if let Some(comments) = last_comments {
            let users: Vec<String> = comments.iter()
                .map(|c| format!("{}「{}」", c.user, c.text))
                .collect();
            format!("【直前のチャット履歴】\n{}", users.join("\n"))
        } else {
            "【直前のチャット履歴】\n(なし)".to_string()
        };

        let final_system_prompt = format!(r#"
{system_instruction}

【今回の演者リスト】
{personas_prompt}

【出力ルール】
1. 必ずJSON形式 `{{ "comments": [ {{ "user": "名前", "text": "発言" }} ... ] }}` で出力すること。
2. "user" はリストの名前を正確に使うこと。
"#, system_instruction = system_instruction, personas_prompt = personas_prompt);

        let user_message = format!("【配信者の発言】\n「{}」\n\n{}", message, context);

        // ... (APIリクエスト作成、送信部分は変更なし) ...
        
        let request_body = GroqRequest {
            model: model_id.to_string(),
            messages: vec![
                Message { role: "system".to_string(), content: final_system_prompt },
                Message { role: "user".to_string(), content: user_message },
            ],
            response_format: ResponseFormat { format_type: "json_object".to_string() },
            max_tokens: 1024,
        };

        let response = self.client
            .post("https://api.groq.com/openai/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        // ... (エラーハンドリング) ...
        if !response.status().is_success() {
             let status = response.status();
             let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
             tracing::error!("[Chat API] Groq API error ({}): {}", status, error_text);
             anyhow::bail!("Groq API returned error: {}", error_text);
        }

        let response_text_raw = response.text().await?;
        let groq_response: GroqResponse = serde_json::from_str(&response_text_raw)
            .map_err(|e| anyhow::anyhow!("Failed to parse GroqResponse: {}", e))?;
        
        let content_json_str = &groq_response.choices[0].message.content;
        let wrapper: CommentsWrapper = serde_json::from_str(content_json_str)
            .map_err(|e| anyhow::anyhow!("Failed to parse content JSON: {}", e))?;

        // 色付けロジック (Tierに基づき決定)
        let mut final_comments = Vec::new();
        for raw in wrapper.comments {
            let persona = active_personas.iter().find(|p| p.name == raw.user);
            
            let color_class = if let Some(p) = persona {
                p.color_class.clone() // 定義された色クラスをそのまま使う
            } else {
                "text-slate-400".to_string()
            };

            final_comments.push(ChatComment {
                user: raw.user,
                text: raw.text,
                color: color_class, 
            });
        }

        Ok(final_comments)
    }
}