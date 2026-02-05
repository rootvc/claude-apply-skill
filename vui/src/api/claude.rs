use serde::{Deserialize, Serialize};

pub struct Client {
    api_key: String,
    http: reqwest::Client,
}

#[derive(Serialize)]
struct Request {
    model: String,
    max_tokens: u32,
    system: String,
    messages: Vec<Message>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tools: Vec<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Message {
    pub role: String,
    pub content: Content,
}

/// Content can be a plain string or an array of content blocks
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum Content {
    Text(String),
    Blocks(Vec<ContentBlock>),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum ContentBlock {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "tool_use")]
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
    },
    #[serde(rename = "tool_result")]
    ToolResult {
        tool_use_id: String,
        content: String,
    },
}

#[derive(Deserialize)]
struct Response {
    content: Vec<ContentBlock>,
    stop_reason: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SendResult {
    pub text: String,
    pub tool_uses: Vec<ToolUse>,
    pub stop_reason: String,
}

#[derive(Debug, Clone)]
pub struct ToolUse {
    pub id: String,
    pub name: String,
    pub input: serde_json::Value,
}

const SYSTEM: &str = include_str!("../../../prompt.txt");

impl Client {
    pub fn new(api_key: String) -> Self {
        let http = reqwest::Client::new();
        Self { api_key, http }
    }

    pub async fn send(
        &self,
        messages: &[Message],
        tools: &[serde_json::Value],
    ) -> Result<SendResult, String> {
        let request = Request {
            model: "claude-sonnet-4-20250514".to_string(),
            max_tokens: 512,
            system: SYSTEM.to_string(),
            messages: messages.to_vec(),
            tools: tools.to_vec(),
        };

        let res = self
            .http
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !res.status().is_success() {
            let error_text = res.text().await.unwrap_or_default();
            return Err(format!("Claude error: {}", error_text));
        }

        let response: Response = res.json().await.map_err(|e| e.to_string())?;

        let mut text_parts = Vec::new();
        let mut tool_uses = Vec::new();

        for block in &response.content {
            match block {
                ContentBlock::Text { text } => text_parts.push(text.clone()),
                ContentBlock::ToolUse { id, name, input } => {
                    tool_uses.push(ToolUse {
                        id: id.clone(),
                        name: name.clone(),
                        input: input.clone(),
                    });
                }
                ContentBlock::ToolResult { .. } => {}
            }
        }

        Ok(SendResult {
            text: text_parts.join(""),
            tool_uses,
            stop_reason: response.stop_reason.unwrap_or_else(|| "end_turn".into()),
        })
    }
}

impl Message {
    pub fn user(content: &str) -> Self {
        Self {
            role: "user".to_string(),
            content: Content::Text(content.to_string()),
        }
    }

    pub fn assistant_blocks(blocks: Vec<ContentBlock>) -> Self {
        Self {
            role: "assistant".to_string(),
            content: Content::Blocks(blocks),
        }
    }

    pub fn tool_result(tool_use_id: &str, result: &str) -> Self {
        Self {
            role: "user".to_string(),
            content: Content::Blocks(vec![ContentBlock::ToolResult {
                tool_use_id: tool_use_id.to_string(),
                content: result.to_string(),
            }]),
        }
    }
}
