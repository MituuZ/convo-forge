use crate::api::{anthropic_client::AnthropicClient, ollama_client::OllamaClient};

pub mod anthropic_client;
pub mod ollama_client;

pub trait ChatApi {
    fn generate_response(
        &self,
        history_messages_json: serde_json::Value,
        user_prompt: &str,
        context_content: Option<&str>,
    ) -> std::io::Result<String>;

    fn model_context_size(&self) -> Option<usize>;

    fn update_system_prompt(&mut self, system_prompt: String);
}

pub fn get_implementation(
    provider: &str,
    model: String,
    system_prompt: String,
    max_tokens: usize,
) -> Box<dyn ChatApi> {
    match provider.to_lowercase().as_str() {
        "anthropic" => Box::new(AnthropicClient::new(model, system_prompt, max_tokens)),
        "ollama" => Box::new(OllamaClient::new(model, system_prompt)),
        _ => panic!("Unsupported provider"),
    }
}
