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
mod models;
pub mod tool;
pub mod traits;
mod user_input;
mod session;

#[cfg(test)]
mod test_support;

use crate::command::commands::{create_command_registry, CommandResult};
use crate::config::{args::Args, AppConfig};
use crate::models::context_file::ContextFile;
use crate::session::Session;
use crate::traits::estimate_context_size::ContextEstimation;
use clap::Parser;
use colored::Colorize;
use command::processor::CommandProcessor;
use std::io::{self};

fn main() -> io::Result<()> {
    let args = Args::parse();
    let app_config = AppConfig::load_config(&args.history_file);
    let command_registry = create_command_registry(app_config.user_config.command_prefixes.clone());
    let mut context_file_path = args.context_file.clone();
    let mut session = Session::new(app_config);

    session.history_file.print_content();
    session.config.print_model_info();

    let mut rebuild_chat_client = false;

    loop {
        if rebuild_chat_client {
            session.rebuild_client();
            rebuild_chat_client = false;
        }

        // TODO: This shouldn't be printed on every iteration and model information should be fetched once
        if session.config.current_profile.provider == "ollama" && session.client.model_supports_tools() {
            println!("Model supports tools");
        }

        let context_file = ContextFile::new(&context_file_path);

        if session.config.user_config.token_estimation {
            print_token_usage(
                session.history_file.estimate_context_size() + context_file.estimate_context_size(),
                &session.client.model_context_size(),
            );
        }

        println!(
            "\nEnter your prompt or a command (type ':q' to end or ':help' for other command)"
        );

        let mut rl = match session.config.create_rustyline_editor(&command_registry) {
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
            &mut session.client,
            &mut session.history_file,
            &mut session.config,
            &command_registry,
            &mut context_file_path,
            &mut rebuild_chat_client,
            context_file.content.clone(),
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
fn print_token_usage(estimated_tokens: usize, maybe_context_size: &Option<usize>) {
    let context_size = match maybe_context_size {
        Some(c) => *c,
        None => {
            return;
        }
    };

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
