//! AI provider abstraction scaffold.
#![allow(dead_code)] // Many items intentionally unused until AI integrations are wired fully.
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatChunk { pub content: String, pub done: bool }

#[async_trait]
pub trait AiProvider: Send + Sync {
    fn name(&self) -> &'static str;
    async fn chat_stream(&self, prompt: &str, sink: &mut dyn ChunkSink) -> Result<(), String>;
}

#[async_trait]
pub trait ChunkSink: Send {
    async fn emit(&mut self, chunk: ChatChunk);
}

pub struct FauxProvider;
#[async_trait]
impl AiProvider for FauxProvider {
    fn name(&self) -> &'static str { "faux" }
    async fn chat_stream(&self, prompt: &str, sink: &mut dyn ChunkSink) -> Result<(), String> {
        let text = format!("Faux: {prompt}");
        for part in text.as_bytes().chunks(12) {
            sink.emit(ChatChunk { content: String::from_utf8_lossy(part).to_string(), done: false }).await;
        }
        sink.emit(ChatChunk { content: String::new(), done: true }).await;
        Ok(())
    }
}

// Registry
pub struct ProviderRegistry { providers: Vec<Arc<dyn AiProvider>>, active: String }

impl ProviderRegistry {
    pub fn new() -> Self {
        let mut reg = Self { providers: vec![Arc::new(FauxProvider)], active: "faux".into() };
        reg.load_active();
        reg
    }
    pub fn list(&self) -> Vec<String> { self.providers.iter().map(|p| p.name().to_string()).collect() }
    pub fn set_active(&mut self, name: &str) -> bool { if self.providers.iter().any(|p| p.name()==name) { self.active=name.into(); let _=self.save_active(); true } else { false } }
    pub fn active(&self) -> Option<Arc<dyn AiProvider>> { self.providers.iter().find(|p| p.name()==self.active).cloned() }
    fn prefs_path() -> Option<std::path::PathBuf> { dirs_next::config_dir().map(|p| p.join("aeonmi").join("ai_prefs.json")) }
    fn load_active(&mut self) {
        if let Some(path) = Self::prefs_path() {
            if let Ok(txt) = std::fs::read_to_string(path) {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&txt) {
                    if let Some(a) = v.get("active").and_then(|x| x.as_str()) {
                        if self.providers.iter().any(|p| p.name() == a) { self.active = a.into(); }
                    }
                }
            }
        }
    }
    fn save_active(&self) -> std::io::Result<()> { if let Some(path)=Self::prefs_path() { if let Some(dir)=path.parent() { std::fs::create_dir_all(dir)?; } let v=serde_json::json!({"active": self.active}); std::fs::write(path, serde_json::to_string_pretty(&v).unwrap_or("{}".into()))?; } Ok(()) }
}
