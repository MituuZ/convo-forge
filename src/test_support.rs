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

#![cfg(test)]

use crate::api::{ChatClient, ChatResponse};
use serde_json::Value;
use std::io;

pub struct TestMockClient {
    system_prompt: String,
}

impl Default for TestMockClient {
    fn default() -> Self {
        Self { system_prompt: String::new() }
    }
}

impl ChatClient for TestMockClient {
    fn generate_response(&self, _history_messages_json: Value, _user_prompt: &str, _context_content: Option<&str>) -> io::Result<ChatResponse> {
        Ok(ChatResponse { content: String::new(), tool_calls: None })
    }

    fn generate_tool_response(&self, _tool_prompt: Value) -> io::Result<ChatResponse> { unreachable!() }

    fn model_context_size(&self) -> Option<usize> { None }

    fn model_supports_tools(&self) -> bool { false }

    fn update_system_prompt(&mut self, system_prompt: String) { self.system_prompt = system_prompt; }

    fn system_prompt(&self) -> String { self.system_prompt.clone() }
}

pub fn make_mock_client() -> Box<dyn ChatClient> {
    Box::new(TestMockClient::default())
}

pub fn make_mock_client_with_prompt<S: Into<String>>(prompt: S) -> Box<dyn ChatClient> {
    Box::new(TestMockClient { system_prompt: prompt.into() })
}
