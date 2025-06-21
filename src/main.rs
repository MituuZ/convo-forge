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

mod command_complete;
mod commands;
mod config;
mod history_file;
mod ollama_client;

use config::Config;

use crate::commands::CommandResult::SwitchHistory;
use crate::commands::{CommandParams, create_command_registry};
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
    let config = Config::load()?;
    let args = Args::parse();
    let command_registry = create_command_registry();

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

    // Get the filename from arguments
    let mut history = HistoryFile::new(args.history_file.clone(), config.cforge_dir.clone())?;
    println!("{}", history.get_content());
    println!("You're conversing with {} model", &config.model);
    let mut ollama_client = OllamaClient::new(config.model.clone(), config.system_prompt.clone());

    match ollama_client.verify() {
        Ok(s) => println!("{}", s),
        Err(e) => {
            println!("\n\nModel is not available: {}", e);
            println!(
                "Check that Ollama is installed or run `ollama pull {}` to pull the model.",
                config.model
            );

            std::process::exit(1);
        }
    }

    let model_context_size =
        OllamaClient::get_model_context_size(&config.model).unwrap_or_else(|e| {
            eprintln!("Error getting model context size: {}", e);
            None
        });

    loop {
        // Calculate and visualize token usage compared to model context size
        if let Some(context_size) = model_context_size {
            let estimated_tokens = history.estimate_token_count();
            let percentage = (estimated_tokens as f64 / context_size as f64 * 100.0).min(100.0);

            // Create a visual representation of token usage
            let bar_width = 50;
            let filled_width = (percentage / 100.0 * bar_width as f64) as usize;
            let empty_width = bar_width - filled_width;

            let bar = format!(
                "[{}{}] {:.1}% ({} / {} tokens)",
                "=".repeat(filled_width),
                " ".repeat(empty_width),
                percentage,
                estimated_tokens,
                context_size
            );

            println!("\n\nEstimated history token usage: {}", bar);
        }

        println!(
            "\n\nEnter your prompt or a command (type ':q' to end or ':help' for other commands)"
        );

        let mut rl = match config.create_editor() {
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

        let user_prompt = user_prompt.trim();
        if user_prompt.starts_with(":") {
            let parts: Vec<&str> = user_prompt.split_whitespace().collect();
            let command_string = parts[0].to_lowercase();
            let args: Vec<&str> = parts[1..].to_vec();

            let command_params =
                CommandParams::new(&args, &mut ollama_client, &mut history, &config.cforge_dir);

            if let Some(command) = command_registry.get(command_string.as_str()) {
                match command.execute(command_params)? {
                    commands::CommandResult::Quit => break,
                    SwitchHistory(new_file) => {
                        history = HistoryFile::new(new_file, config.cforge_dir.clone())?;
                        println!("{}", history.get_content());
                        println!("Switched to history file: {}", history.filename);
                        continue;
                    }
                    commands::CommandResult::Continue => continue,
                }
            } else {
                println!("Unknown command: {}", command_string);
                continue;
            }
        }

        let history_json = match history.get_content_json() {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Error reading history file: {}", e);
                break;
            }
        };

        let ollama_response = ollama_client.generate_response(
            history_json,
            user_prompt,
            input_file_content.as_deref(),
        )?;

        history.append_user_input(user_prompt)?;

        // Print the AI response with the delimiter to make it easier to parse
        println!("{}", history.append_ai_response(&ollama_response)?);
    }

    Ok(())
}
