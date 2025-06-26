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
mod user_input;

use config::Config;

use crate::commands::CommandResult::SwitchHistory;
use crate::commands::{create_command_registry, CommandParams};
use crate::history_file::HistoryFile;
use crate::ollama_client::OllamaClient;
use crate::user_input::UserInput;
use clap::Parser;
use colored::Colorize;
use std::fs::{self};
use std::io::{self};
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to file containing chat history. Can be either relative (to `cforge_dir`) or absolute.
    /// If not provided, the last history file will be used, which is saved in `~/.cforge.toml`.
    history_file: Option<String>,

    /// Optional file with content to be used as input for each chat message
    #[arg(short = 'f', long = "file")]
    context_file: Option<PathBuf>,
}

fn main() -> io::Result<()> {
    let mut config = Config::load()?;
    let args = Args::parse();
    let command_registry = create_command_registry();

    // Read the input file if provided
    let context_file_content = if let Some(file_path) = args.context_file {
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

    let history_path = args.history_file.unwrap_or_else(|| {
        match config.last_history_file.clone() {
            Some(path) => path,
            None => {
                eprintln!("No history file specified and no previous history file found.");
                println!(
                    "You must specify a history file `cforge <histoty_file>` for the first time."
                );
                println!("See `cforge --help` for more information.");
                std::process::exit(1);
            }
        }
    });

    config.update_last_history_file(history_path.clone())?;

    let mut history = HistoryFile::new(history_path.clone(), config.cforge_dir.clone())?;
    println!("{}", history.get_content());
    println!("\n\nYou're conversing with {} model", &config.model);
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
        if config.token_estimation {
            print_token_usage(
                history.estimate_token_count(),
                model_context_size.unwrap_or(0),
            );
        }

        println!(
            "\nEnter your prompt or a command (type ':q' to end or ':help' for other commands)"
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

        match UserInput::parse(&user_prompt) {
            UserInput::Command(command) => {
                let command_params = CommandParams::new(
                    command.args,
                    &mut ollama_client,
                    &mut history,
                    &config.cforge_dir,
                );

                if let Some(command) = command_registry.get(&command.name) {
                    match command.execute(command_params)? {
                        commands::CommandResult::Quit => break,
                        SwitchHistory(new_file) => {
                            history = HistoryFile::new(new_file, config.cforge_dir.clone())?;
                            config.update_last_history_file(history.filename.clone())?;
                            println!("{}", history.get_content());
                            println!("Switched to history file: {}", history.filename);
                            continue;
                        }
                        commands::CommandResult::Continue => continue,
                    }
                } else {
                    println!("Unknown command: {}", command.name);
                    continue;
                }
            }
            UserInput::Prompt(prompt) => {
                let history_json = match history.get_content_json() {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("Error reading history file: {}", e);
                        break;
                    }
                };

                let ollama_response = ollama_client.generate_response(
                    history_json,
                    &prompt,
                    context_file_content.as_deref(),
                )?;

                history.append_user_input(&prompt)?;

                // Print the AI response with the delimiter to make it easier to parse
                println!("{}", history.append_ai_response(&ollama_response)?);
            }
        }
    }

    Ok(())
}

/// Calculate and visualize token usage compared to model context size
fn print_token_usage(estimated_tokens: usize, context_size: usize) {
    let percentage = (estimated_tokens as f64 / context_size as f64 * 100.0).min(100.0);

    let bar_width = 50;
    let filled_width = (percentage / 100.0 * bar_width as f64) as usize;
    let empty_width = bar_width - filled_width;

    let filled_bar = if percentage < 50.0 {
        "=".repeat(filled_width).green()
    } else if percentage < 75.0 {
        "=".repeat(filled_width).yellow()
    } else {
        "=".repeat(filled_width).red()
    };

    let bar = format!(
        "[{}{}] {:.1}% ({} / {} tokens)",
        filled_bar,
        " ".repeat(empty_width),
        percentage,
        estimated_tokens,
        context_size
    );

    println!(
        "\n\nEstimated history token usage (1 token ≈ 4 characters): {}",
        bar
    );
}
