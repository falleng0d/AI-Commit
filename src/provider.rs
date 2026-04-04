use anyhow::{Context, Result, anyhow, bail};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

use crate::config::Config;

pub struct AiClient {
    client: Client,
    endpoint: String,
    models_endpoint: String,
    api_key: String,
    model: String,
}

impl AiClient {
    pub fn new(config: &Config) -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .context("failed to build HTTP client")?;

        Ok(Self {
            client,
            endpoint: format!("{}/chat/completions", config.host),
            models_endpoint: format!("{}/models", config.host),
            api_key: config.api_key.clone(),
            model: config.model.clone(),
        })
    }

    pub fn generate_commit_message(&self, prompt: &str) -> Result<String> {
        let model = self.resolve_model()?;
        let body = ChatCompletionRequest {
            model,
            temperature: 0.1,
            messages: vec![
                ChatMessage {
                    role: "system".to_string(),
                    content:
                        "Generate semantic git commit messages. Return only the commit message."
                            .to_string(),
                },
                ChatMessage {
                    role: "user".to_string(),
                    content: prompt.to_string(),
                },
            ],
        };

        let response = self
            .client
            .post(&self.endpoint)
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .context("failed to send chat completion request")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().unwrap_or_default();
            bail!("AI provider request failed with {status}: {body}");
        }

        let payload: ChatCompletionResponse = response
            .json()
            .context("failed to decode chat completion response")?;

        let message = payload
            .choices
            .into_iter()
            .next()
            .ok_or_else(|| anyhow!("AI provider returned no choices"))?
            .message
            .content;

        Ok(sanitize_commit_message(&message))
    }

    fn resolve_model(&self) -> Result<String> {
        let response = self
            .client
            .get(&self.models_endpoint)
            .bearer_auth(&self.api_key)
            .send()
            .context("failed to query AI provider models")?;

        if !response.status().is_success() {
            return Ok(self.model.clone());
        }

        let payload: ModelsResponse = response
            .json()
            .context("failed to decode AI provider models response")?;

        if payload.data.iter().any(|entry| entry.id == self.model) {
            return Ok(self.model.clone());
        }

        for preferred in [
            "qwen-3-235b-a22b-instruct-2507",
            "llama3.1-8b",
            "gpt-oss-120b",
        ] {
            if payload.data.iter().any(|entry| entry.id == preferred) {
                return Ok(preferred.to_string());
            }
        }

        payload
            .data
            .into_iter()
            .next()
            .map(|entry| entry.id)
            .ok_or_else(|| anyhow!("AI provider returned no models"))
    }
}

#[derive(Debug, Serialize)]
struct ChatCompletionRequest {
    model: String,
    temperature: f32,
    messages: Vec<ChatMessage>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Debug, Deserialize)]
struct ModelsResponse {
    data: Vec<ModelEntry>,
}

#[derive(Debug, Deserialize)]
struct ModelEntry {
    id: String,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatMessage,
}

fn sanitize_commit_message(message: &str) -> String {
    let line = message.lines().next().unwrap_or_default().trim();
    line.trim_matches('`').trim_matches('"').to_string()
}

#[cfg(test)]
mod tests {
    use super::sanitize_commit_message;

    #[test]
    fn strips_quotes_and_code_ticks() {
        assert_eq!(
            sanitize_commit_message("`feat: add tool`\nextra"),
            "feat: add tool"
        );
        assert_eq!(
            sanitize_commit_message("\"fix: patch bug\""),
            "fix: patch bug"
        );
    }
}
