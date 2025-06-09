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

use std::io::{BufReader, Read, Write};
use std::process::{Command, Stdio};
use std::time::Duration;
use std::{io, thread};

pub(crate) struct OllamaClient {
    model: String,
    pub(crate) system_prompt: String,
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
        // Create the ollama command with stdout piped
        let mut cmd = Command::new("ollama")
            .args(&["run", &self.model])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()?;

        // Create the input for the ollama process
        if let Some(mut stdin) = cmd.stdin.take() {
            // First, add the system prompt
            stdin.write_all(b"Here is the system prompt: ")?;
            stdin.write_all(self.system_prompt.as_bytes())?;

            // Then add the context file content if available
            if let Some(ref content) = context_content {
                stdin.write_all(b"\n\nAdditional context from file: ")?;
                stdin.write_all(content.as_bytes())?;
            }

            // Then include the full history file for context
            stdin.write_all(b"\n\nPrevious conversation: ")?;
            stdin.write_all(history_content.as_bytes())?;

            // Finally, add the user prompt
            stdin.write_all(b"\n\nCurrent user prompt: ")?;
            stdin.write_all(user_prompt.as_bytes())?;
        }

        let stdout = cmd.stdout.take().expect("Failed to open stdout");
        let mut reader = BufReader::new(stdout);
        let full_response =
            read_process_output_with_interrupt(&mut reader).expect("error reading process output");
        let ollama_response = String::from_utf8_lossy(&full_response).to_string();

        Ok(ollama_response)
    }

    pub(crate) fn update_system_prompt(&mut self, new_system_prompt: String) {
        self.system_prompt = new_system_prompt;
    }
}

fn read_process_output_with_interrupt(reader: &mut BufReader<impl Read>) -> io::Result<Vec<u8>> {
    let mut buffer = [0; 1024];
    let mut full_response = Vec::new();

    loop {
        // Set up non-blocking read with timeout
        match reader.read(&mut buffer) {
            Ok(0) => break, // End of stream
            Ok(bytes_read) => {
                // Write the chunk to console
                io::stdout().write_all(&buffer[..bytes_read])?;
                io::stdout().flush()?;

                // Store the chunk for later file writing
                full_response.extend_from_slice(&buffer[..bytes_read]);
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                // Would block, just wait a bit and try again
                thread::sleep(Duration::from_millis(10));
                continue;
            }
            Err(e) => return Err(e),
        }

        // Small delay to reduce CPU usage and allow interrupt checking
        thread::sleep(Duration::from_millis(10));
    }

    Ok(full_response)
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
