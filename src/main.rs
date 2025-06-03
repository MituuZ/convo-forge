use std::env;
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};
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

    // Prompt the user for input
    println!("Enter your prompt:");
    let mut user_prompt = String::new();
    io::stdin().read_line(&mut user_prompt)?;

    // Append the user prompt to the file
    file.write_all(user_prompt.as_bytes())?;

    // Update file_content to include the newly added prompt
    file_content.push_str(&user_prompt);

    // Create the ollama command with stdout captured
    let mut cmd = Command::new("ollama")
        .args(&["run", "gemma3:12b"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()?;

    // Write to stdin
    if let Some(mut stdin) = cmd.stdin.take() {
        stdin.write_all(file_content.as_bytes())?;
        // stdin is closed automatically when it goes out of scope
    }

    // Wait for the command to complete and get the output
    let output = cmd.wait_with_output()?;

    if !output.status.success() {
        eprintln!("ollama command failed with exit code: {:?}", output.status.code());
        std::process::exit(output.status.code().unwrap_or(1));
    }

    // Convert the output bytes to a string
    let ollama_response = String::from_utf8_lossy(&output.stdout);
    
    // Print the response to console
    println!("{}", ollama_response);
    
    // Reopen the file in append mode to add the response
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open(filename)?;
        
    // Append a delimiter before the response
    file.write_all(b"\n\n--- AI Response ---\n\n")?;
    
    // Append ollama's response to the file
    file.write_all(ollama_response.as_bytes())?;

    Ok(())
}