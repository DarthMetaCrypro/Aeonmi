use anyhow::{Result, anyhow, bail};
use super::AiProvider;
use std::time::Duration;

#[derive(Default)]
pub struct Copilot;

#[derive(serde::Serialize)]
struct ChatRequest<'a> { messages: Vec<ChatMessage<'a>> }
#[derive(serde::Serialize)]
struct ChatMessage<'a> { role: &'a str, content: &'a str }

#[derive(serde::Deserialize, Debug)]
struct ChatResponse { choices: Vec<Choice> }
#[derive(serde::Deserialize, Debug)]
struct Choice { message: ChoiceMessage }
#[derive(serde::Deserialize, Debug)]
struct ChoiceMessage { content: String }

impl AiProvider for Copilot {
    fn name(&self) -> &'static str { "copilot" }
    fn chat(&self, prompt: &str) -> Result<String> {
        let trimmed = prompt.trim();
        if trimmed.is_empty() { bail!("empty prompt"); }
        let key = std::env::var("GITHUB_COPILOT_TOKEN")
            .map_err(|_| anyhow!("GITHUB_COPILOT_TOKEN not set"))?;
        // Endpoint placeholder; actual Copilot APIs may differ or require websocket; adjust when official public API is used.
        let endpoint = std::env::var("AEONMI_COPILOT_ENDPOINT").unwrap_or_else(|_| "https://api.githubcopilot.com/v1/chat/completions".to_string());
        let req = ChatRequest { messages: vec![ChatMessage { role: "user", content: trimmed }] };
        let client = reqwest::blocking::Client::builder().timeout(Duration::from_secs(45)).build()?;
        let resp = client.post(&endpoint)
            .bearer_auth(&key)
            .header("Content-Type", "application/json")
            .json(&req)
            .send()?;
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().unwrap_or_default();
            bail!("copilot http error {status}: {text}");
        }
        let cr: ChatResponse = resp.json()?;
        let content = cr.choices.first()
            .map(|c| c.message.content.trim().to_string())
            .filter(|s| !s.is_empty())
            .ok_or_else(|| anyhow!("empty response"))?;
        Ok(content)
    }
}
