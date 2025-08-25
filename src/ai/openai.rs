use anyhow::{Result, anyhow, bail};
use super::AiProvider;
use std::time::Duration;

#[derive(Default)]
pub struct OpenAi;

#[derive(serde::Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    messages: Vec<ChatMessage<'a>>,
    temperature: f32,
    #[serde(skip_serializing_if = "Option::is_none")] stream: Option<bool>,
}

#[derive(serde::Serialize)]
struct ChatMessage<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(serde::Deserialize, Debug)]
struct ChatResponse {
    choices: Vec<Choice>,
}
#[derive(serde::Deserialize, Debug)]
struct Choice { message: ChoiceMessage }
#[derive(serde::Deserialize, Debug)]
struct ChoiceMessage { content: String }

impl AiProvider for OpenAi {
    fn name(&self) -> &'static str { "openai" }
    fn chat(&self, prompt: &str) -> Result<String> {
        let trimmed = prompt.trim();
        if trimmed.is_empty() { bail!("empty prompt"); }
        let key = std::env::var("OPENAI_API_KEY")
            .map_err(|_| anyhow!("OPENAI_API_KEY not set in environment"))?;
        let model = std::env::var("AEONMI_OPENAI_MODEL").unwrap_or_else(|_| "gpt-4o-mini".to_string());
    let req = ChatRequest { model: &model, messages: vec![ChatMessage { role: "user", content: trimmed }], temperature: 0.7, stream: None };
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(45))
            .build()?;
        let resp = client
            .post("https://api.openai.com/v1/chat/completions")
            .bearer_auth(&key)
            .header("Content-Type", "application/json")
            .json(&req)
            .send()?;
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().unwrap_or_default();
            bail!("openai http error {status}: {text}");
        }
        let cr: ChatResponse = resp.json()?;
        let content = cr.choices.first()
            .map(|c| c.message.content.trim().to_string())
            .filter(|s| !s.is_empty())
            .ok_or_else(|| anyhow!("empty response"))?;
        Ok(content)
    }
}

impl OpenAi {
    fn stream_chat(&self, prompt: &str, cb: &mut dyn FnMut(&str)) -> Result<()> {
        let trimmed = prompt.trim();
        if trimmed.is_empty() { bail!("empty prompt"); }
        let key = std::env::var("OPENAI_API_KEY")
            .map_err(|_| anyhow!("OPENAI_API_KEY not set in environment"))?;
        let model = std::env::var("AEONMI_OPENAI_MODEL").unwrap_or_else(|_| "gpt-4o-mini".to_string());
        let req = ChatRequest { model: &model, messages: vec![ChatMessage { role: "user", content: trimmed }], temperature: 0.7, stream: Some(true) };
        let client = reqwest::blocking::Client::builder().timeout(Duration::from_secs(120)).build()?;
        let resp = client.post("https://api.openai.com/v1/chat/completions")
            .bearer_auth(&key)
            .header("Content-Type", "application/json")
            .json(&req)
            .send()?;
        if !resp.status().is_success() { let status = resp.status(); let text = resp.text().unwrap_or_default(); bail!("openai http error {status}: {text}"); }
        use std::io::{BufRead, BufReader};
        let mut reader = BufReader::new(resp);
        let mut line = String::new();
        loop {
            line.clear();
            let n = reader.read_line(&mut line)?; if n == 0 { break; }
            let trimmed_line = line.trim_start();
            if !trimmed_line.starts_with("data:") { continue; }
            let payload = trimmed_line.trim_start_matches("data:").trim();
            if payload == "[DONE]" { break; }
            #[derive(serde::Deserialize)]
            struct StreamObj { choices: Vec<StreamChoice> }
            #[derive(serde::Deserialize)]
            struct StreamChoice { delta: Option<Delta> }
            #[derive(serde::Deserialize)]
            struct Delta { content: Option<String> }
            if let Ok(obj) = serde_json::from_str::<StreamObj>(payload) {
                if let Some(ch) = obj.choices.first() { if let Some(d) = &ch.delta { if let Some(c) = &d.content { cb(c); } } }
            }
        }
        Ok(())
    }
}

impl super::AiProvider for OpenAi {
    fn chat_stream(&self, prompt: &str, cb: &mut dyn FnMut(&str)) -> Result<()> { self.stream_chat(prompt, cb) }
}
