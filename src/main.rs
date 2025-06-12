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

mod commands;
mod config;
mod history_file;
mod ollama_client;

use config::Config;

use crate::commands::CommandResult::SwitchHistory;
use crate::commands::{create_command_registry, CommandParams};
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
    let mut history = HistoryFile::new(args.history_file.clone(), config.sllama_dir.clone())?;
    println!("{}", history.get_content());
    println!("You're conversing with {} model", &config.model);
    let mut ollama_client = OllamaClient::new(config.model.clone(), config.system_prompt.clone());
    println!("Press Enter during AI generation to interrupt the response.");

    // Main conversation loop
    loop {
        // Prompt the user for input
        println!(
            "\nEnter your prompt or a command (type ':q' to end or ':help' for other commands)"
        );

        let rustyline_config = config.create_rustyline_config();
        let mut rl = match rustyline::DefaultEditor::with_config(rustyline_config) {
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

            if user_prompt.starts_with(":") {
                let command_params = CommandParams::new(
                    &*args,
                    &mut ollama_client,
                    &mut history,
                    &config.sllama_dir,
                );

                if let Some(command_fn) = command_registry.get(command_string.as_str()) {
                    match command_fn(command_params)? {
                        commands::CommandResult::Quit => break,
                        SwitchHistory(new_file) => {
                            history = HistoryFile::new(new_file, config.sllama_dir.clone())?;
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
        }

        let ollama_response = ollama_client.generate_response(
            history.get_content(),
            &user_prompt,
            input_file_content.as_deref(),
        )?;

        history.append_user_input(&user_prompt)?;

        history.append_ai_response(&ollama_response)?;
    }

    Ok(())
}
