use crossterm::event;
use crossterm::event::{Event, KeyCode};
use std::io::{BufReader, Read, Write};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, TryRecvError};
use std::time::Duration;
use std::{io, thread};

pub(crate) struct OllamaClient {
    model: String,
    system_prompt: String,
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
        context_content: Option<&str>
    ) -> io::Result<(String, bool)> {
        // Create the ollama command with stdout piped
        let mut cmd = Command::new("ollama")
            .args(&["run", &self.model])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()?;

        // Create the input for the ollama process
        if let Some(mut stdin) = cmd.stdin.take() {
            // Use the full history for context (including current user input)
            stdin.write_all(history_content.as_bytes())?;

            // Include the context file content if available
            if let Some(ref content) = context_content {
                stdin.write_all(b"\n\nAdditional context from file: ")?;
                stdin.write_all(content.as_bytes())?;
            }

            // Finally, add the system prompt
            stdin.write_all(b"\n\n")?;
            stdin.write_all(self.system_prompt.as_bytes())?;
        }

        // Get stdout stream from the child process
        let stdout = cmd.stdout.take().expect("Failed to open stdout");
        let mut reader = BufReader::new(stdout);

        // Set up channel for interrupt signal
        let (interrupt_tx, interrupt_rx) = mpsc::channel();

        thread::spawn(move || {
            println!("\nAI is responding... (Press Enter to interrupt)\n");
            loop {
                if event::poll(Duration::from_millis(100)).unwrap() {
                    if let Event::Key(key) = event::read().unwrap() {
                        if key.code == KeyCode::Enter {
                            let _ = interrupt_tx.send(());
                            break;
                        }
                    }
                }
            }
        });

        // Read response while checking for interrupt
        let (full_response, was_interrupted) =
            read_process_output_with_interrupt(&mut reader, &interrupt_rx, &mut cmd)
                .expect("error reading process output");

        let ollama_response = String::from_utf8_lossy(&full_response).to_string();
        
        Ok((ollama_response, was_interrupted))
    }
}

fn read_process_output_with_interrupt(
    reader: &mut BufReader<impl Read>,
    interrupt_rx: &Receiver<()>,
    cmd: &mut Child
) -> io::Result<(Vec<u8>, bool)> {
    let mut buffer = [0; 1024];
    let mut full_response = Vec::new();
    let mut was_interrupted = false;

    loop {
        // Check for interrupt signal
        match interrupt_rx.try_recv() {
            Ok(_) | Err(TryRecvError::Disconnected) => {
                // Interrupt signal received, kill the process
                println!("\n[Interrupting AI response...]");
                cmd.kill()?;
                was_interrupted = true;
                break;
            }
            Err(TryRecvError::Empty) => {
                // No interrupt, continue reading
            }
        }

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

    // Try to read any remaining output if we were interrupted
    if was_interrupted {
        loop {
            match reader.read(&mut buffer) {
                Ok(0) => break,
                Ok(bytes_read) => {
                    full_response.extend_from_slice(&buffer[..bytes_read]);
                },
                Err(_) => break,
            }
        }
    }

    Ok((full_response, was_interrupted))
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
