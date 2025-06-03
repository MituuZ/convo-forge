use std::env;
use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufReader, Read, Write};
use std::process::{Command, Child, Stdio};
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;
use std::time::Duration;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    // Check if a filename was provided
    if args.len() < 2 {
        eprintln!("Usage: {} <filename>", args[0]);
        std::process::exit(1);
    }

    // Get the filename from arguments
    let filename = &args[1];

    // Open the file in append mode, create it if it doesn't exist
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(filename)?;

    // Read the current file content
    let mut file_content = String::new();
    file.read_to_string(&mut file_content)?;

    println!("Starting conversation. Type 'exit' to end the session.");
    println!("File '{}' will store the entire conversation.", filename);
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

        // Append the user prompt to the file with a delimiter
        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .open(filename)?;
        
        file.write_all(b"\n\n--- User Input ---\n\n")?;
        file.write_all(user_prompt.as_bytes())?;
        
        // Update file_content to include the newly added prompt
        if !file_content.is_empty() && !file_content.ends_with("\n") {
            file_content.push('\n');
        }
        file_content.push_str("\n--- User Input ---\n\n");
        file_content.push_str(&user_prompt);

        // Create the ollama command with stdout piped
        let mut cmd = Command::new("ollama")
            .args(&["run", "gemma3:12b"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()?;

        // Write the full conversation history to stdin
        if let Some(mut stdin) = cmd.stdin.take() {
            stdin.write_all(file_content.as_bytes())?;
            // stdin is closed automatically when it goes out of scope
        }

        // Get stdout stream from the child process
        let stdout = cmd.stdout.take().expect("Failed to open stdout");
        let mut reader = BufReader::new(stdout);
        
        // Set up channel for interrupt signal
        let (tx, rx) = mpsc::channel();
        
        // Spawn a thread to check for user input (interrupt signal)
        thread::spawn(move || {
            println!("\nAI is responding... (Press Enter to interrupt)\n");
            let mut buffer = String::new();
            io::stdin().read_line(&mut buffer).ok();
            tx.send(()).ok(); // Send interrupt signal
        });
        
        // Read response from ollama while checking for interrupt
        let (full_response, was_interrupted) = read_process_output_with_interrupt(&mut reader, &rx, &mut cmd)?;
        
        // Convert the collected response to a string
        let ollama_response = String::from_utf8_lossy(&full_response);
        
        // Add a note if the response was interrupted
        let response_with_note = if was_interrupted {
            format!("{}\n\n[Response was interrupted by user]", ollama_response)
        } else {
            ollama_response.to_string()
        };
        
        // Append the AI response to the file
        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .open(filename)?;
            
        // Append a delimiter before the response
        file.write_all(b"\n\n--- AI Response ---\n\n")?;
        file.write_all(response_with_note.as_bytes())?;
        
        // Update the file_content with the AI response for the next iteration
        if !file_content.is_empty() && !file_content.ends_with("\n") {
            file_content.push('\n');
        }
        file_content.push_str("\n--- AI Response ---\n\n");
        file_content.push_str(&response_with_note);
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