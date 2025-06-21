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
use serde::Deserialize;
use serde_json::Value;
use std::io;

static LLM_PROTOCOL: &str = "http";
static LLM_HOST: &str = "localhost";
static LLM_PORT: &str = "11434";
static LLM_ENDPOINT: &str = "/api/chat";

pub(crate) struct OllamaClient {
    model: String,
    pub(crate) system_prompt: String,
}

#[derive(Deserialize, Debug)]
pub(crate) struct OllamaResponse {
    pub(crate) message: OllamaMessage,
    pub(crate) done: bool,
    pub(crate) done_reason: String,
}

#[derive(Deserialize, Debug)]
pub(crate) struct OllamaMessage {
    pub(crate) content: String,
}

impl OllamaClient {
    pub(crate) fn new(model: String, system_prompt: String) -> Self {
        Self {
            model,
            system_prompt,
        }
    }

    /// Send an empty message to ollama to preload the model.
    pub(crate) fn verify(&self) -> io::Result<String> {
        let send_body = serde_json::json!({
            "model": self.model,
        });

        match Self::send_request_and_handle_response(&send_body) {
            Ok(response) => Ok(response.message.content),
            Err(e) => Err(e),
        }
    }

    pub(crate) fn generate_response(
        &self,
        history_messages_json: Value,
        user_prompt: &str,
        context_content: Option<&str>,
    ) -> io::Result<String> {
        let send_body = Self::build_json_body(
            self.model.as_str(),
            self.system_prompt.as_str(),
            context_content.unwrap_or(""),
            user_prompt,
            &history_messages_json,
        );

        let response = Self::poll_for_response(&send_body)?;
        Ok(response.message.content)
    }

    fn poll_for_response(send_body: &Value) -> io::Result<OllamaResponse> {
        let ollama_response = Self::send_request_and_handle_response(send_body)?;

        if ollama_response.done
            && ollama_response.done_reason == "load"
            && ollama_response.message.content.is_empty()
        {
            println!("Model responded with an empty message. Retrying request...");

            std::thread::sleep(std::time::Duration::from_secs(1));

            return Self::poll_for_response(send_body);
        }

        Ok(ollama_response)
    }

    fn send_request_and_handle_response(send_body: &Value) -> io::Result<OllamaResponse> {
        let mut response = ureq::post(Self::api_url())
            .send_json(send_body)
            .map_err(|e| io::Error::other(e.to_string()))?;

        let ollama_response: OllamaResponse = response
            .body_mut()
            .read_json()
            .map_err(|e| io::Error::other(e.to_string()))?;

        Ok(ollama_response)
    }

    fn build_json_body(
        model: &str,
        system_prompt: &str,
        context_content: &str,
        user_prompt: &str,
        history_messages_json: &Value,
    ) -> Value {
        let messages = Self::create_messages(
            system_prompt,
            context_content,
            user_prompt,
            history_messages_json,
        );

        serde_json::json!({
            "model": model,
            "messages": messages,
            "stream": false,
        })
    }

    fn create_messages(
        system_prompt: &str,
        context_content: &str,
        user_prompt: &str,
        history_messages_json: &Value,
    ) -> Vec<Value> {
        let mut messages = vec![];

        messages.push(serde_json::json!({ "role": "system", "content": system_prompt }));

        if !context_content.is_empty() {
            messages.push(serde_json::json!({ "role": "system", "content": format!("Additional context that the user has provided: {}", context_content) }));
        }

        if let Some(history_messages_json) = history_messages_json.as_array() {
            for message in history_messages_json {
                messages.push(message.clone());
            }
        }

        messages.push(serde_json::json!({ "role": "user", "content": user_prompt }));

        messages
    }

    pub(crate) fn update_system_prompt(&mut self, new_system_prompt: String) {
        self.system_prompt = new_system_prompt;
    }

    fn api_url() -> String {
        format!(
            "{}://{}:{}{}",
            LLM_PROTOCOL, LLM_HOST, LLM_PORT, LLM_ENDPOINT
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_ollama_client_creation() {
        let model = "gemma3:4b".to_string();
        let system_prompt = "You are a helpful assistant.".to_string();

        let client = OllamaClient::new(model.clone(), system_prompt.clone());

        assert_eq!(client.model, model);
        assert_eq!(client.system_prompt, system_prompt);
    }

    #[test]
    fn test_update_system_prompt() {
        let model = "gemma3:4b".to_string();
        let initial_prompt = "Initial prompt".to_string();
        let new_prompt = "New system prompt".to_string();

        let mut client = OllamaClient::new(model, initial_prompt);
        client.update_system_prompt(new_prompt.clone());

        assert_eq!(client.system_prompt, new_prompt);
    }

    #[test]
    fn test_create_messages_basic() {
        let system_prompt = "You are a helpful assistant.";
        let context_content = "";
        let user_prompt = "Hello!";
        let history = json!([]);

        let messages =
            OllamaClient::create_messages(system_prompt, context_content, user_prompt, &history);

        assert_eq!(messages.len(), 2);
        assert_eq!(
            messages[0],
            json!({"role": "system", "content": "You are a helpful assistant."})
        );
        assert_eq!(messages[1], json!({"role": "user", "content": "Hello!"}));
    }

    #[test]
    fn test_create_messages_with_context() {
        let system_prompt = "You are a helpful assistant.";
        let context_content = "This is some context.";
        let user_prompt = "Hello!";
        let history = json!([]);

        let messages =
            OllamaClient::create_messages(system_prompt, context_content, user_prompt, &history);

        assert_eq!(messages.len(), 3);
        assert_eq!(
            messages[0],
            json!({"role": "system", "content": "You are a helpful assistant."})
        );
        assert_eq!(
            messages[1],
            json!({"role": "system", "content": "Additional context that the user has provided: This is some context."})
        );
        assert_eq!(messages[2], json!({"role": "user", "content": "Hello!"}));
    }

    #[test]
    fn test_create_messages_with_history() {
        let system_prompt = "You are a helpful assistant.";
        let context_content = "";
        let user_prompt = "How are you?";
        let history = json!([
            {"role": "user", "content": "Hello!"},
            {"role": "assistant", "content": "Hi there! How can I help you today?"}
        ]);

        let messages =
            OllamaClient::create_messages(system_prompt, context_content, user_prompt, &history);

        assert_eq!(messages.len(), 4);
        assert_eq!(
            messages[0],
            json!({"role": "system", "content": "You are a helpful assistant."})
        );
        assert_eq!(messages[1], json!({"role": "user", "content": "Hello!"}));
        assert_eq!(
            messages[2],
            json!({"role": "assistant", "content": "Hi there! How can I help you today?"})
        );
        assert_eq!(
            messages[3],
            json!({"role": "user", "content": "How are you?"})
        );
    }

    #[test]
    fn test_create_messages_with_context_and_history() {
        let system_prompt = "You are a helpful assistant.";
        let context_content = "User is a developer.";
        let user_prompt = "Can you explain async/await?";
        let history = json!([
            {"role": "user", "content": "Hello!"},
            {"role": "assistant", "content": "Hi there! How can I help you today?"}
        ]);

        let messages =
            OllamaClient::create_messages(system_prompt, context_content, user_prompt, &history);

        assert_eq!(messages.len(), 5);
        assert_eq!(
            messages[0],
            json!({"role": "system", "content": "You are a helpful assistant."})
        );
        assert_eq!(
            messages[1],
            json!({"role": "system", "content": "Additional context that the user has provided: User is a developer."})
        );
        assert_eq!(messages[2], json!({"role": "user", "content": "Hello!"}));
        assert_eq!(
            messages[3],
            json!({"role": "assistant", "content": "Hi there! How can I help you today?"})
        );
        assert_eq!(
            messages[4],
            json!({"role": "user", "content": "Can you explain async/await?"})
        );
    }

    #[test]
    fn test_create_messages_with_invalid_history() {
        let system_prompt = "You are a helpful assistant.";
        let context_content = "";
        let user_prompt = "Hello!";
        let history = json!({"invalid": "not an array"}); // Not an array

        let messages =
            OllamaClient::create_messages(system_prompt, context_content, user_prompt, &history);

        assert_eq!(messages.len(), 2);
        assert_eq!(
            messages[0],
            json!({"role": "system", "content": "You are a helpful assistant."})
        );
        assert_eq!(messages[1], json!({"role": "user", "content": "Hello!"}));
    }

    #[test]
    fn test_create_messages_with_empty_system_prompt() {
        let system_prompt = "";
        let context_content = "";
        let user_prompt = "Hello!";
        let history = json!([]);

        let messages =
            OllamaClient::create_messages(system_prompt, context_content, user_prompt, &history);

        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0], json!({"role": "system", "content": ""}));
        assert_eq!(messages[1], json!({"role": "user", "content": "Hello!"}));
    }

    #[test]
    fn test_build_json_body() {
        let model = "gemma3:4b";
        let system_prompt = "You are a helpful assistant.";
        let context_content = "Sample context";
        let user_prompt = "Hello!";
        let history_messages = json!([{"role": "user", "content": "Previous message"}]);

        let result = OllamaClient::build_json_body(
            model,
            system_prompt,
            context_content,
            user_prompt,
            &history_messages,
        );

        assert_eq!(result["model"], json!(model));
        assert_eq!(result["stream"], json!(false));
        assert!(result.get("messages").is_some());
        assert!(result["messages"].is_array());
        assert_eq!(result["messages"].as_array().unwrap().len(), 4);
    }

    #[test]
    fn test_build_json_body_minimal() {
        let model = "llama3:8b";
        let system_prompt = "Test prompt";
        let context_content = ""; // Empty context
        let user_prompt = "Test question";
        let history_messages = json!([]); // Empty history

        let result = OllamaClient::build_json_body(
            model,
            system_prompt,
            context_content,
            user_prompt,
            &history_messages,
        );

        assert_eq!(result["model"], json!(model));
        assert_eq!(result["stream"], json!(false));
        assert!(result.get("messages").is_some());
        assert!(result["messages"].is_array());
        assert_eq!(result["messages"].as_array().unwrap().len(), 2);
    }
}
