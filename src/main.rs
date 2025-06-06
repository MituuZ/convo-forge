mod config;
mod history_file;
mod ollama_client;

use config::Config;

use crate::history_file::HistoryFile;
use crate::ollama_client::OllamaClient;
use clap::Parser;
use std::fs::{self};
use std::io::{self};
use std::path::PathBuf;

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
    let ollama_client = OllamaClient::new(config.model, config.system_prompt);

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

        let (ollama_response, was_interrupted) = ollama_client.generate_response(
            history.get_content(),
            input_file_content.as_deref(),
        )?;

        history.append_ai_response(&ollama_response, was_interrupted)?;
    }

    Ok(())
}
