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

use serde::{Deserialize, Serialize};
use std::io;

pub(crate) struct OllamaClient {
    model: String,
    pub(crate) system_prompt: String,
}

#[derive(Serialize)]
pub(crate) struct OllamaRequest {
    pub(crate) message_history: String,
    pub(crate) current_prompt: String,
    pub(crate) context: Option<String>,
    pub(crate) system_prompt: String,
}

#[derive(Deserialize, Debug)]
pub(crate) struct OllamaResponse {
    pub(crate) model: String,
    pub(crate) created_at: String,
    pub(crate) message: OllamaMessage,
    pub(crate) done: bool,
}

#[derive(Deserialize, Debug)]
pub(crate) struct OllamaMessage {
    pub(crate) role: String,
    pub(crate) content: String,
}

impl OllamaClient {
    pub(crate) fn new(model: String, system_prompt: String) -> Self {
        Self {
            model,
            system_prompt,
        }
    }

    pub(crate) fn generate_response(
        &self,
        history_content: &str,
        user_prompt: &str,
        context_content: Option<&str>,
    ) -> io::Result<String> {
        let send_body = serde_json::json!({
            "model": self.model,
            "messages": [
                { "role": "system", "content": self.system_prompt },
                { "role": "system", "content": format!("Additional context that the user has provided: {}", context_content.unwrap_or("")) },
                { "role": "system", "content": format!("The conversation so far: {}", history_content) },
                { "role": "user", "content": user_prompt },
                ],
            "stream": false,
        });

        let mut response = ureq::post("http://localhost:11434/api/chat")
            .send_json(&send_body)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

        let ollama_response: OllamaResponse = response
            .body_mut()
            .read_json()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

        Ok(ollama_response.message.content)
    }

    pub(crate) fn update_system_prompt(&mut self, new_system_prompt: String) {
        self.system_prompt = new_system_prompt;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ollama_client_creation() {
        let model = "llama2".to_string();
        let system_prompt = "You are a helpful assistant.".to_string();

        let client = OllamaClient::new(model.clone(), system_prompt.clone());

        assert_eq!(client.model, model);
        assert_eq!(client.system_prompt, system_prompt);
    }
}
