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
 */
use crate::api::{anthropic_client::AnthropicClient, ollama_client::OllamaClient};
use serde::Deserialize;
use std::fmt::{Display, Formatter};

pub mod anthropic_client;
mod client_util;
pub mod ollama_client;

#[derive(Deserialize, Debug)]
pub struct ChatResponse {
    pub content: String,
    pub tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Deserialize, Debug)]
pub struct ToolCall {
    pub(crate) function: Function,
}

#[derive(Deserialize, Debug)]
pub(crate) struct Function {
    pub(crate) name: String,
    pub(crate) arguments: serde_json::Value,
}

impl Display for ToolCall {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Function: {}, Arguments: {}",
            self.function.name, self.function.arguments
        )
    }
}

pub trait ChatClient {
    fn generate_response(
        &self,
        history_messages_json: serde_json::Value,
        user_prompt: &str,
        context_content: Option<&str>,
    ) -> std::io::Result<ChatResponse>;

    fn model_context_size(&self) -> Option<usize>;

    fn update_system_prompt(&mut self, system_prompt: String);

    fn system_prompt(&self) -> String;
}

pub fn get_chat_client_implementation(
    provider: &str,
    model: &str,
    system_prompt: String,
    max_tokens: usize,
) -> Box<dyn ChatClient> {
    match provider.to_lowercase().as_str() {
        "anthropic" => Box::new(AnthropicClient::new(
            model.to_string(),
            system_prompt,
            max_tokens,
        )),
        "ollama" => {
            let mut client = OllamaClient::new(model.to_string(), system_prompt);
            client.verify();
            Box::new(client)
        }
        _ => panic!("Unsupported provider"),
    }
}
