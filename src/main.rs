mod config;
mod history_file;

use config::Config;

use crate::history_file::HistoryFile;
use clap::Parser;
use crossterm::event::{self, Event, KeyCode};
use std::fs::{self};
use std::io::{self, BufReader, Read, Write};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;
use std::time::Duration;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to file containing chat history. Can be either relative (to `sslama_dir`) or absolute.
    history_file: String,

    /// Optional file with content to be used as input for each chat message
    #[arg(short = 'f', long = "file")]
    input_file: Option<PathBuf>,
}

fn main() -> io::Result<()> {
    let config = match Config::load() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error loading config: {}", e);
            return Ok(());
        }   
    };

    // Parse command-line arguments
    let args = Args::parse();

    // Get the filename from arguments
    let filename = &args.history_file;

    // Read the input file if provided
    let input_file_content = if let Some(file_path) = args.input_file {
        match fs::read_to_string(file_path.clone()) {
            Ok(content) => {
                println!("Loaded input from file: {}", file_path.display());
                Some(content)
            }
            Err(e) => {
                eprintln!("Error reading input file: {}", e);
                None
            }
        }
    } else {
        None
    };
    
    let mut history = HistoryFile::new(filename.clone(), config.sllama_dir.clone())?;

    println!("Starting conversation. Type 'exit' to end the session.");
    println!("Press Enter during AI generation to interrupt the response.");
    
    // Main conversation loop
    loop {
        // Prompt the user for input
        println!("\nEnter your prompt (or type 'exit' to end):");
        let mut user_prompt = String::new();
        io::stdin().read_line(&mut user_prompt)?;

        // Check if user wants to exit
        if user_prompt.trim().to_lowercase() == "exit" {
            println!("Ending conversation. All interactions saved to '{}'", filename);
            break;
        }

        history.append_user_input(&user_prompt)?;

        // Create the ollama command with stdout piped
        let mut cmd = Command::new("ollama")
            .args(&["run", &config.model])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()?;

        // Create the input for the ollama process
        if let Some(mut stdin) = cmd.stdin.take() {
            // Use the full history for context (including current user input)
            stdin.write_all(history.get_content().as_bytes())?;

            // Include the context file content if available
            if let Some(ref content) = input_file_content {
                stdin.write_all(b"\n\nAdditional context from file: ")?;
                stdin.write_all(content.as_bytes())?;
            }
            
            // Finally, add the system prompt
            stdin.write_all(b"\n\n")?;
            stdin.write_all(config.system_prompt.as_bytes())?;
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

        let ollama_response = String::from_utf8_lossy(&full_response);
        
        history.append_ai_response(&ollama_response, was_interrupted)?;
    }

    Ok(())
}

// Function to read process output while checking for interrupt signal
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