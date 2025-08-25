use anyhow::{Result, anyhow, bail};
use super::AiProvider;
use std::time::Duration;

#[derive(Default)]
pub struct Perplexity;

#[derive(serde::Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    messages: Vec<ChatMessage<'a>>,
    temperature: f32,
}

#[derive(serde::Serialize)]
struct ChatMessage<'a> { role: &'a str, content: &'a str }

#[derive(serde::Deserialize, Debug)]
struct ChatResponse { choices: Vec<Choice> }
#[derive(serde::Deserialize, Debug)]
struct Choice { message: ChoiceMessage }
#[derive(serde::Deserialize, Debug)]
struct ChoiceMessage { content: String }

impl AiProvider for Perplexity {
    fn name(&self) -> &'static str { "perplexity" }
    fn chat(&self, prompt: &str) -> Result<String> {
        let trimmed = prompt.trim();
        if trimmed.is_empty() { bail!("empty prompt"); }
        let key = std::env::var("PERPLEXITY_API_KEY")
            .map_err(|_| anyhow!("PERPLEXITY_API_KEY not set"))?;
        let model = std::env::var("AEONMI_PERPLEXITY_MODEL").unwrap_or_else(|_| "llama-3.1-sonar-small-chat".to_string());
        let req = ChatRequest { model: &model, messages: vec![ChatMessage { role: "user", content: trimmed }], temperature: 0.7 };
        let client = reqwest::blocking::Client::builder().timeout(Duration::from_secs(45)).build()?;
        let resp = client.post("https://api.perplexity.ai/chat/completions")
            .bearer_auth(&key)
            .header("Content-Type", "application/json")
            .json(&req)
            .send()?;
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().unwrap_or_default();
            bail!("perplexity http error {status}: {text}");
        }
        let cr: ChatResponse = resp.json()?;
        let content = cr.choices.first()
            .map(|c| c.message.content.trim().to_string())
            .filter(|s| !s.is_empty())
            .ok_or_else(|| anyhow!("empty response"))?;
        Ok(content)
    }
}
