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
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Deserialize)]
struct Response {
    content: Vec<Content>,
}

#[derive(Deserialize)]
struct Content {
    text: String,
}

const SYSTEM: &str = include_str!("../../../prompt.txt");

impl Client {
    pub fn new(api_key: String) -> Self {
        let http = reqwest::Client::new();
        Self { api_key, http }
    }

    pub async fn send(&self, messages: &[Message]) -> Result<String, String> {
        let request = Request {
            model: "claude-sonnet-4-20250514".to_string(),
            max_tokens: 256,
            system: SYSTEM.to_string(),
            messages: messages.to_vec(),
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

        let text = response
            .content
            .first()
            .map(|b| b.text.clone())
            .unwrap_or_default();

        Ok(text)
    }
}

impl Message {
    pub fn user(content: &str) -> Self {
        Self {
            role: "user".to_string(),
            content: content.to_string(),
        }
    }

    pub fn assistant(content: &str) -> Self {
        Self {
            role: "assistant".to_string(),
            content: content.to_string(),
        }
    }
}
