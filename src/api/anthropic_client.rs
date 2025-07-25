use serde_json::Value;
use std::{env, io};

use crate::api::ChatApi;

static LLM_PROTOCOL: &str = "https";
static LLM_HOST: &str = "api.anthropic.com";
static LLM_ENDPOINT: &str = "/v1/messages";

pub struct AnthropicClient {
    model: String,
    system_prompt: String,
    max_tokens: usize,
}

impl ChatApi for AnthropicClient {
    fn generate_response(
        &self,
        history_messages_json: Value,
        user_prompt: &str,
        context_content: Option<&str>,
    ) -> io::Result<String> {
        let send_body = Self::build_json_body(
            &self.model,
            &self.system_prompt,
            context_content.unwrap_or(""),
            user_prompt,
            &history_messages_json,
            self.max_tokens,
        );

        let response = Self::send_request_and_handle_response(&send_body)?;
        Ok(response)
    }

    fn model_context_size(&self) -> Option<usize> {
        None
    }

    fn update_system_prompt(&mut self, system_prompt: String) {
        self.system_prompt = system_prompt;
    }
}

impl AnthropicClient {
    pub fn new(model: String, system_prompt: String, max_tokens: usize) -> Self {
        Self {
            model,
            system_prompt,
            max_tokens,
        }
    }

    fn send_request_and_handle_response(send_body: &Value) -> io::Result<String> {
        let mut response = ureq::post(Self::api_url())
            .header("x-api-key", &Self::get_api_key()?)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .send_json(send_body)
            .map_err(|e| io::Error::other(e.to_string()))?;

        let response: serde_json::Value = response
            .body_mut()
            .read_json()
            .map_err(|e| io::Error::other(e.to_string()))?;

        let message = response
            .get("content")
            .and_then(|v| v.get(0))
            .and_then(|v| v.get("text"))
            .and_then(|v| v.as_str());

        let asd = match message {
            Some(message) => message.to_string(),
            None => "No response".to_string(),
        };

        Ok(asd)
    }

    fn api_url() -> String {
        format!("{LLM_PROTOCOL}://{LLM_HOST}{LLM_ENDPOINT}")
    }

    fn build_json_body(
        model: &str,
        system_prompt: &str,
        context_content: &str,
        user_prompt: &str,
        history_messages_json: &Value,
        max_tokens: usize,
    ) -> Value {
        let messages = Self::create_messages(
            system_prompt,
            context_content,
            user_prompt,
            history_messages_json,
        );

        serde_json::json!({
            "model": model,
            "max_tokens": max_tokens,
            "messages": messages,
        })
    }

    fn create_messages(
        system_prompt: &str,
        context_content: &str,
        user_prompt: &str,
        history_messages_json: &Value,
    ) -> Vec<Value> {
        let mut messages = vec![];

        messages.push(serde_json::json!({ "role": "assistant", "content": system_prompt }));

        if !context_content.is_empty() {
            messages.push(serde_json::json!({ "role": "user", "content": format!("Additional context that should be considered: {}", context_content) }));
        }

        if let Some(history_messages_json) = history_messages_json.as_array() {
            for message in history_messages_json {
                messages.push(message.clone());
            }
        }

        messages.push(serde_json::json!({ "role": "user", "content": user_prompt }));

        messages
    }

    fn get_api_key() -> io::Result<String> {
        env::var("ANTHROPIC_API_KEY")
            .map_err(|_| io::Error::other("Missing ANTHROPIC_API_KEY env var"))
    }
}
