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
        "ollama" => {
            let mut client = OllamaClient::new(model, system_prompt);
            client.verify();
            Box::new(client)
        }
        _ => panic!("Unsupported provider"),
    }
}
