/*
 * Copyright © 2025 Mitja Leino
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated
 * documentation files (the “Software”), to deal in the Software without restriction, including without limitation
 * the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software,
 * and to permit persons to whom the Software is furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE
 * WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS
 * OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT,
 * TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
 *
 */
use serde_json::Value;
use std::{env, io};

use crate::api::{ChatApi, client_util::create_messages};

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
        let messages = create_messages(
            &self.system_prompt,
            context_content.unwrap_or(""),
            user_prompt,
            &history_messages_json,
            "assistant",
        );

        let send_body = Self::build_json_body(&self.model, self.max_tokens, messages);

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

    fn build_json_body(model: &str, max_tokens: usize, messages: Vec<Value>) -> Value {
        serde_json::json!({
            "model": model,
            "max_tokens": max_tokens,
            "messages": messages,
        })
    }

    fn get_api_key() -> io::Result<String> {
        env::var("ANTHROPIC_API_KEY")
            .map_err(|_| io::Error::other("Missing ANTHROPIC_API_KEY env var"))
    }
}
