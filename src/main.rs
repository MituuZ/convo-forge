use std::env;
use std::fs::File;
use std::io::{self, Read};
use std::process::{Command, Stdio};

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    // Check if a filename was provided
    if args.len() < 2 {
        eprintln!("Usage: {} <input_filename>", args[0]);
        std::process::exit(1);
    }

    // Get the input filename from arguments
    let input_filename = &args[1];

    // Open the input file
    let mut input_file = File::open(input_filename)?;

    // Read the file content
    let mut file_content = String::new();
    input_file.read_to_string(&mut file_content)?;

    // Create the ollama command
    let mut cmd = Command::new("ollama")
        .args(&["run", "gemma3:12b"])
        .stdin(Stdio::piped())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;

    // Write file content to ollama's stdin
    if let Some(mut stdin) = cmd.stdin.take() {
        use std::io::Write;
        stdin.write_all(file_content.as_bytes())?;
        // stdin is closed automatically when it goes out of scope
    }

    // Wait for the command to complete
    let status = cmd.wait()?;

    if !status.success() {
        eprintln!("ollama command failed with exit code: {:?}", status.code());
        std::process::exit(status.code().unwrap_or(1));
    }

    Ok(())
}
