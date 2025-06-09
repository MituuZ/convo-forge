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

mod config;
mod history_file;
mod ollama_client;

use config::Config;
use std::env;

use crate::history_file::HistoryFile;
use crate::ollama_client::OllamaClient;
use clap::Parser;
use std::fs::{self};
use std::io::{self};
use std::path::PathBuf;
use std::process::Command;

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
    let mut filename: String = args.history_file.clone();

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
    println!("{}", history.get_content());
    println!("You're conversing with {} model", &config.model);
    let mut ollama_client = OllamaClient::new(config.model, config.system_prompt);
    println!("Press Enter during AI generation to interrupt the response.");

    // Main conversation loop
    loop {
        // Prompt the user for input
        println!(
            "\nEnter your prompt or a command (type ':q' to end or ':help' for other commands)"
        );
        let mut rl = match rustyline::DefaultEditor::new() {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Error initializing rustyline: {}", e);
                break;
            }
        };
        let readline = rl.readline(">> ");
        let user_prompt = match readline {
            Ok(line) => line,
            Err(e) => {
                eprintln!("Error reading input: {}", e);
                break;
            }
        };

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
                        // Update filename, so ending conversation prints the correct filename
                        filename = new_history_file;
                        history = HistoryFile::new(filename.clone(), config.sllama_dir.clone())?;
                        println!("{}", history.get_content());
                        println!("Switched to history file: {}", filename);
                    }
                    continue;
                }
                ":sysprompt" => {
                    ollama_client.update_system_prompt(args.join(" "));
                    continue;
                }
                ":help" => {
                    println!("\nAvailable commands:");
                    println!(":q - quit");
                    println!(
                        ":list <optional pattern> - list files in the sllama directory. \
                    Optionally, you can provide a pattern to filter the results."
                    );
                    println!(
                        ":switch <history_file> - switch to a different history file. \
                    Either relative to sllama_dir or absolute path."
                    );
                    println!(":help - show this help message");
                    println!(":edit - open the history file in your editor");
                    println!(":sysprompt <prompt> - set the system prompt for current session");
                    continue;
                }
                ":edit" => {
                    let editor = env::var("EDITOR")
                        .or_else(|_| env::var("VISUAL"))
                        .unwrap_or_else(|_| {
                            if cfg!(target_os = "windows") {
                                "notepad".to_string()
                            } else {
                                "vi".to_string()
                            }
                        });

                    let status = Command::new(editor).arg(history.path.clone()).status()?;

                    if !status.success() {
                        println!("Error opening file in editor");
                    }

                    history.reload_content();
                    continue;
                }
                _ => {
                    println!("Unknown command '{}'", command_string);
                    continue;
                }
            }
        }

        let (ollama_response, was_interrupted) = ollama_client.generate_response(
            history.get_content(),
            &user_prompt,
            input_file_content.as_deref(),
        )?;

        history.append_user_input(&user_prompt)?;

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
