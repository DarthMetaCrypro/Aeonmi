//! AI Mother Module skeleton: multi-provider abstraction.
use anyhow::Result;

pub trait AiProvider: Send + Sync {
    fn name(&self) -> &'static str;
    fn chat(&self, prompt: &str) -> Result<String>;
    fn chat_stream(&self, _prompt: &str, _cb: &mut dyn FnMut(&str)) -> Result<()> {
        // Default fallback: call non-streaming and emit once
        let full = self.chat(_prompt)?;
        _cb(&full);
        Ok(())
    }
}

#[cfg(feature = "ai-openai")]
pub mod openai;
#[cfg(feature = "ai-copilot")]
pub mod copilot;
#[cfg(feature = "ai-perplexity")]
pub mod perplexity;
#[cfg(feature = "ai-deepseek")]
pub mod deepseek;

pub struct AiRegistry {
    providers: Vec<Box<dyn AiProvider>>,
}

impl AiRegistry {
    #[allow(unused_mut)]
    pub fn new() -> Self {
        let mut r = Self { providers: Vec::new() };
        #[cfg(feature = "ai-openai")]
        { r.providers.push(Box::new(openai::OpenAi::default())); }
        #[cfg(feature = "ai-copilot")]
        { r.providers.push(Box::new(copilot::Copilot::default())); }
        #[cfg(feature = "ai-perplexity")]
        { r.providers.push(Box::new(perplexity::Perplexity::default())); }
        #[cfg(feature = "ai-deepseek")]
        { r.providers.push(Box::new(deepseek::DeepSeek::default())); }
    r
    }
    pub fn list(&self) -> Vec<&'static str> { self.providers.iter().map(|p| p.name()).collect() }
    pub fn first(&self) -> Option<&Box<dyn AiProvider>> { self.providers.first() }
    pub fn get(&self, name: &str) -> Option<&dyn AiProvider> {
        self.providers.iter().find(|p| p.name() == name).map(|b| b.as_ref())
    }
}
