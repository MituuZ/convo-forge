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
use serde::Deserialize;
use serde_json::Value;
use std::fmt::Display;
use std::io;
use std::process::Command;

use crate::api::client_util::create_messages;
use crate::api::{ChatClient, ChatResponse, ToolCall};

static LLM_PROTOCOL: &str = "http";
static LLM_HOST: &str = "localhost";
static LLM_PORT: &str = "11434";
static LLM_ENDPOINT: &str = "/api/chat";

struct ModelInformation {
    model: String,
    context_size: Option<usize>,
    supports_tools: bool,
}

pub struct OllamaClient {
    pub(crate) system_prompt: String,
    model_information: ModelInformation,
}

#[derive(Deserialize, Debug)]
pub(crate) struct OllamaResponse {
    pub(crate) message: OllamaMessage,
    pub(crate) done: bool,
    pub(crate) done_reason: String,
    // pub(crate) error: Option<String>, TODO: Check if this can be used
}

#[derive(Deserialize, Debug)]
pub(crate) struct OllamaMessage {
    pub(crate) content: String,
    pub(crate) tool_calls: Option<Vec<ToolCall>>,
}

impl ChatClient for OllamaClient {
    fn generate_response(
        &self,
        history_messages_json: Value,
        user_prompt: &str,
        context_content: Option<&str>,
    ) -> io::Result<ChatResponse> {
        let messages = create_messages(
            &self.system_prompt,
            context_content.unwrap_or(""),
            user_prompt,
            &history_messages_json,
            "system",
        );

        let send_body = Self::build_json_body(&self.model_information, messages);

        let response = Self::poll_for_response(&send_body)?;
        Ok(ChatResponse {
            content: response.message.content,
            tool_calls: response.message.tool_calls,
        })
    }

    fn model_context_size(&self) -> Option<usize> {
        self.model_information.context_size
    }

    fn update_system_prompt(&mut self, new_system_prompt: String) {
        self.system_prompt = new_system_prompt;
    }

    fn system_prompt(&self) -> String {
        self.system_prompt.to_string()
    }
}

impl OllamaClient {
    /// Create the client and verify that it is responding
    pub fn new(model: String, system_prompt: String) -> Self {
        Self {
            system_prompt,
            model_information: ModelInformation {
                model: model.clone(),
                context_size: None,
                supports_tools: false,
            },
        }
    }

    pub fn verify(&mut self) {
        match self.preload() {
            Ok(s) => println!("{s}"),
            Err(e) => {
                println!("\n\nModel is not available: {e}");
                panic!(
                    "Failed to verify ollama client\nCheck that Ollama is installed or run `ollama pull {}` to pull the model.",
                    &self.model_information.model
                );
            }
        }

        if let Ok(model_info) = Self::get_model_information(&self.model_information.model) {
            self.model_information = model_info;
        } else {
            eprintln!("Error getting model information");
        }
    }

    /// Send an empty message to ollama to preload the model.
    fn preload(&self) -> io::Result<String> {
        let send_body = serde_json::json!({
            "model": self.model_information.model,
        });

        match Self::send_request_and_handle_response(&send_body) {
            Ok(response) => Ok(response.message.content),
            Err(e) => Err(e),
        }
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

    fn build_json_body(model_information: &ModelInformation, messages: Vec<Value>) -> Value {
        let mut base_body = serde_json::json!({
            "model": model_information.model,
            "messages": messages,
            "stream": false,
        });

        if model_information.supports_tools {
            let tools = serde_json::json!([
                {
                    "type": "function",
                    "function": {
                        "name": "get_weather",
                        "description": "Always tell the user current weather",
                        "parameters": {
                            "type": "object",
                            "properties": {
                                "location": {"type": "string"}
                            },
                            "required": ["location"]
                        }
                    }
                }
            ]);
            base_body
                .as_object_mut()
                .unwrap()
                .insert("tools".to_string(), tools);
        }

        base_body
    }

    fn api_url() -> String {
        format!("{LLM_PROTOCOL}://{LLM_HOST}:{LLM_PORT}{LLM_ENDPOINT}")
    }

    /// Gets the context size and tool support information for a specific model by executing the `ollama show [model]` command.
    fn get_model_information(model_name: &str) -> Result<ModelInformation, io::Error> {
        let output = Command::new("ollama")
            .arg("show")
            .arg(model_name)
            .output()
            .map_err(|e| io::Error::other(format!("Failed to execute command: {e}")))?;

        if !output.status.success() {
            let error_message = String::from_utf8_lossy(&output.stderr);
            return Err(io::Error::other(format!("Command failed: {error_message}")));
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        Ok(Self::parse_model_information(&output_str, model_name))
    }

    fn parse_model_information(output: &str, model_name: &str) -> ModelInformation {
        let mut supports_tools = false;
        let mut context_size = None;
        let mut passed_capabilites = false;

        // Look for the line containing "context length" in the Model section
        for line in output.lines() {
            let line = line.trim();
            if line.contains("context length") {
                // Extract the number at the end of the line
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    // The context length should be the last part
                    if let Ok(parsed_context_size) = parts.last().unwrap().parse::<usize>() {
                        context_size = Some(parsed_context_size);
                    }
                }
            } else if line.contains("Capabilities") {
                passed_capabilites = true;
            } else if passed_capabilites && line.contains("tools") {
                supports_tools = true;
            }
        }

        ModelInformation {
            model: model_name.to_string(),
            context_size,
            supports_tools,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ollama_client_creation() {
        let model = "gemma3:4b".to_string();
        let system_prompt = "You are a helpful assistant.".to_string();

        let client = OllamaClient::new(model.clone(), system_prompt.clone());

        assert_eq!(client.model_information.model, model);
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
    fn test_parse_context_size() {
        // Test with the example output from the issue description
        let example_output = r#"  Model
    architecture        gemma3    
    parameters          4.3B      
    context length      131072    
    embedding length    2560      
    quantization        Q4_K_M    

  Capabilities
    completion    
    vision        

  Parameters
    stop           "<end_of_turn>"    
    temperature    1                  
    top_k          64                 
    top_p          0.95               

  License
    Gemma Terms of Use                  
    Last modified: February 21, 2024    
    ..."#;

        let context_size = OllamaClient::parse_model_information(example_output, "").context_size;
        assert_eq!(context_size, Some(131072));
    }

    #[test]
    fn test_parse_context_size_with_different_format() {
        // Test with a slightly different format
        let different_format = r#"Model
    architecture: gemma3    
    parameters: 4.3B      
    context length: 131072    
    embedding length: 2560"#;

        let context_size = OllamaClient::parse_model_information(different_format, "").context_size;
        assert_eq!(context_size, Some(131072));
    }

    #[test]
    fn test_parse_context_size_not_found() {
        // Test with output that doesn't contain context length
        let no_context_length = r#"Model
    architecture        gemma3    
    parameters          4.3B      
    embedding length    2560      
    quantization        Q4_K_M"#;

        let context_size =
            OllamaClient::parse_model_information(no_context_length, "").context_size;
        assert_eq!(context_size, None);
    }

    #[test]
    fn test_parse_context_size_invalid_format() {
        // Test with invalid format for context length
        let invalid_format = r#"Model
    architecture        gemma3    
    parameters          4.3B      
    context length      invalid    
    embedding length    2560"#;

        let context_size = OllamaClient::parse_model_information(invalid_format, "").context_size;
        assert_eq!(context_size, None);
    }

    #[test]
    fn test_parse_tools_supported() {
        // Test with invalid format for context length
        let invalid_format = r#"Model
    architecture        gemma3
    parameters          4.3B
    context length      invalid
    embedding length    2560
  Capabilities
    completion
    tools
    "#;

        let tools_supported =
            OllamaClient::parse_model_information(invalid_format, "").supports_tools;
        assert!(tools_supported);
    }

    #[test]
    fn test_parse_tools_not_supported() {
        // Test with invalid format for context length
        let invalid_format = r#"Model
    architecture        gemma3
    parameters          4.3B
    context length      invalid
    embedding length    2560
  Capabilities
    completion
    "#;

        let tools_supported =
            OllamaClient::parse_model_information(invalid_format, "").supports_tools;
        assert!(!tools_supported);
    }

    #[test]
    fn test_parse_tools_not_supported_alt_format() {
        // Test with invalid format for context length
        let invalid_format = r#"Model
    architecture        gemma3
    parameters          4.3B
    context length      invalid
    embedding length    2560
    tools
  Capabilities
    completion
    "#;

        let tools_supported =
            OllamaClient::parse_model_information(invalid_format, "").supports_tools;
        assert!(!tools_supported);
    }
}
