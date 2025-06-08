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

    println!("Press Enter during AI generation to interrupt the response.");

    // Main conversation loop
    loop {
        // Prompt the user for input
        println!("\nEnter your prompt (or type ':q' to end):");
        let mut user_prompt = String::new();
        io::stdin().read_line(&mut user_prompt)?;

        // Check if user wants to exit
        let user_prompt = user_prompt.trim().to_lowercase();
        if user_prompt.starts_with(":") {
            let parts: Vec<&str> = user_prompt.split_whitespace().collect();
            let command_string = parts[0].to_lowercase();
            let args: Vec<&str> = parts[1..].to_vec();

            match command_string.as_str() {
                ":q" => {
                    println!(
                        "Ending conversation. All interactions saved to '{}'",
                        filename
                    );
                    break;
                }
                ":list" => {
                    list_command(&config.sllama_dir, args);
                    continue;
                }
                ":switch" => {
                    if let Some(new_history_file) = switch_command(args) {
                        let filename = new_history_file;
                        history = HistoryFile::new(filename.clone(), config.sllama_dir.clone())?;
                        println!("{}", history.get_content());
                        println!("Switched to history file: {}", filename);
                    }
                    continue;
                }
                _ => {
                    println!("Unknown command '{}'", command_string);
                    continue;
                }
            }
        }

        history.append_user_input(&user_prompt)?;

        let (ollama_response, was_interrupted) = ollama_client
            .generate_response(history.get_content(), input_file_content.as_deref())?;

        history.append_ai_response(&ollama_response, was_interrupted)?;
    }

    Ok(())
}

fn list_command(sllama_dir: &str, args: Vec<&str>) {
    let pattern = args.get(0).unwrap_or(&"");

    fn list_dir_contents(dir: &str, pattern: &str, sllama_dir: &str) -> io::Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if (pattern.is_empty() || path.display().to_string().contains(pattern))
                && !path.is_dir()
            {
                match path.display().to_string().strip_prefix(sllama_dir) {
                    None => println!("{}", path.display()),
                    Some(ds) => {
                        let mut cleaned_ds = ds.to_string();
                        if cleaned_ds.starts_with('/') {
                            cleaned_ds = cleaned_ds[1..].to_string();
                        }
                        println!("{}", cleaned_ds)
                    }
                }
            }
            if path.is_dir() {
                list_dir_contents(path.to_str().unwrap(), pattern, sllama_dir)?;
            }
        }
        Ok(())
    }

    match list_dir_contents(sllama_dir, pattern, sllama_dir) {
        Ok(_) => (),
        Err(e) => eprintln!("Error reading directory: {}", e),
    }
}

fn switch_command(args: Vec<&str>) -> Option<String> {
    let new_history_file = args.get(0).unwrap_or(&"");

    if new_history_file.is_empty() {
        println!("Error: No history file specified. Usage: :switch <history_file>");
        return None;
    }

    Some(new_history_file.to_string())
}
