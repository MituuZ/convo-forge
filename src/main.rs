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
 */

pub mod api;
mod command;
pub mod config;
mod history_file;
pub mod tool;
mod user_input;

#[cfg(test)]
mod test_support;

use crate::api::{get_chat_client_implementation, ChatClient};
use crate::command::commands::{create_command_registry, CommandResult};
use crate::config::AppConfig;
use crate::history_file::HistoryFile;
use clap::Parser;
use colored::Colorize;
use command::processor::CommandProcessor;
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
    let mut app_config = AppConfig::load_config();
    let args = Args::parse();
    let command_registry = create_command_registry(app_config.user_config.command_prefixes.clone());
    let mut context_file_path = args.context_file.clone();

    let history_path = args.history_file.unwrap_or_else(|| {
        match app_config.cache_config.last_history_file.clone() {
            Some(path) => path,
            None => {
                println!(
                    "You must specify a history file `cforge <history_file>` for the first time."
                );
                println!("See `cforge --help` for more information.");
                panic!("No history file specified and no previous history file found.");
            }
        }
    });

    app_config.update_last_history_file(history_path.clone());

    let mut history = HistoryFile::new(
        history_path.clone(),
        app_config.data_dir.display().to_string(),
    )?;
    println!("{}", history.get_content());
    println!(
        "\n\nYou're conversing with model '{}' ({}) from profile '{}'",
        &app_config.current_model,
        &app_config.current_model.model_type,
        &app_config.current_profile.name
    );

    let mut chat_client: Box<dyn ChatClient> = get_chat_client_implementation(
        &app_config.current_profile.provider,
        &app_config.current_model.model,
        app_config.user_config.system_prompt.clone(),
        app_config.user_config.max_tokens,
    );
    let mut rebuild_chat_client = false;

    loop {
        if rebuild_chat_client {
            chat_client = get_chat_client_implementation(
                &app_config.current_profile.provider,
                &app_config.current_model.model,
                app_config.user_config.system_prompt.clone(),
                app_config.user_config.max_tokens,
            );
            rebuild_chat_client = false;
        }

        // TODO: This shouldn't be printed on every iteration and model information should be fetched once
        if &app_config.current_profile.provider == "ollama" && chat_client.model_supports_tools() {
            println!("Model supports tools");
        }

        // Read the context file if provided
        let context_file_content = if let Some(file_path) = &context_file_path {
            match fs::read_to_string(file_path.clone()) {
                Ok(content) => Some(content),
                Err(e) => {
                    eprintln!("Error reading context file: {e}");
                    None
                }
            }
        } else {
            None
        };

        if let Some(model_context_size) = chat_client.model_context_size()
            && app_config.user_config.token_estimation
        {
            print_token_usage(
                estimate_token_count(history.get_content())
                    + estimate_token_count(context_file_content.as_deref().unwrap_or("")),
                model_context_size,
            );
        }

        println!(
            "\nEnter your prompt or a command (type ':q' to end or ':help' for other command)"
        );

        let mut rl = match app_config.create_rustyline_editor(&command_registry) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Error initializing rustyline: {e}");
                break;
            }
        };
        let readline = rl.readline(">> ");
        let user_prompt = match readline {
            Ok(line) => line,
            Err(e) => {
                eprintln!("Error reading input: {e}");
                break;
            }
        };

        let mut processor = CommandProcessor::new(
            &mut chat_client,
            &mut history,
            &mut app_config,
            &command_registry,
            &mut context_file_path,
            &mut rebuild_chat_client,
            context_file_content.clone(),
        );

        match processor.process(&user_prompt) {
            Ok(CommandResult::Quit) => break,
            Err(e) => {
                eprintln!("Error processing input: {e}");
                break;
            }
            _ => continue,
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

    println!("\n\nEstimated token usage (1 token ≈ 4 characters): {bar}");
}

fn estimate_token_count(prompt: &str) -> usize {
    let char_count = prompt.chars().count();
    char_count / 4 + 1 // Add 1 to avoid returning 0 for very short content
}
