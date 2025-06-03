use std::env;
use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufReader, Read, Write};
use std::process::{Command, Stdio};

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
        let mut buffer = [0; 1024];
        let mut full_response = Vec::new();

        println!("\nAI is responding...\n");

        // Read stdout in chunks and stream to console while collecting the full response
        loop {
            let bytes_read = reader.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            
            // Write the chunk to console
            io::stdout().write_all(&buffer[..bytes_read])?;
            io::stdout().flush()?;
            
            // Store the chunk for later file writing
            full_response.extend_from_slice(&buffer[..bytes_read]);
        }

        // Wait for the command to complete
        let status = cmd.wait()?;

        if !status.success() {
            eprintln!("\nollama command failed with exit code: {:?}", status.code());
            continue; // Continue the loop even if this request failed
        }

        // Convert the collected response to a string
        let ollama_response = String::from_utf8_lossy(&full_response);
        
        // Append the AI response to the file
        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .open(filename)?;
            
        // Append a delimiter before the response
        file.write_all(b"\n\n--- AI Response ---\n\n")?;
        file.write_all(ollama_response.as_bytes())?;
        
        // Update the file_content with the AI response for the next iteration
        if !file_content.is_empty() && !file_content.ends_with("\n") {
            file_content.push('\n');
        }
        file_content.push_str("\n--- AI Response ---\n\n");
        file_content.push_str(&ollama_response);
    }

    Ok(())
}