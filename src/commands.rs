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
use crate::history_file::HistoryFile;
use crate::ollama_client::OllamaClient;
use std::collections::HashMap;
use std::env::args;
use std::process::Command;
use std::{env, fs, io};

pub enum CommandResult {
    Continue,
    Quit,
    SwitchHistory(String),
}

pub struct CommandParams<'a, 'b> {
    args: &'a [&'b str],
    ollama_client: &'a mut OllamaClient,
    history: &'a mut HistoryFile,
    sllama_dir: &'a str,
}

impl<'a, 'b> CommandParams<'a, 'b> {
    pub fn new(
        args: &'a [&'b str],
        ollama_client: &'a mut OllamaClient,
        history: &'a mut HistoryFile,
        sllama_dir: &'a str,
    ) -> Self {
        CommandParams {
            args,
            ollama_client,
            history,
            sllama_dir,
        }
    }
}

type CommandFn = fn(CommandParams) -> io::Result<CommandResult>;

pub fn create_command_registry() -> HashMap<&'static str, CommandFn> {
    let mut commands: HashMap<&'static str, CommandFn> = HashMap::new();

    commands.insert(":q", quit_command);
    commands.insert(":list", list_command);
    commands.insert(":switch", switch_command);
    commands.insert(":sysprompt", sysprompt_command);
    commands.insert(":help", help_command);
    commands.insert(":edit", edit_command);

    commands
}

fn quit_command(command_params: CommandParams) -> io::Result<CommandResult> {
    println!(
        "Ending conversation. All interactions saved to '{}'",
        command_params.history.filename
    );
    Ok(CommandResult::Quit)
}

fn list_command(command_params: CommandParams) -> io::Result<CommandResult> {
    let pattern = command_params.args.get(0).unwrap_or(&"");

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

    list_dir_contents(
        command_params.sllama_dir,
        pattern,
        command_params.sllama_dir,
    )?;
    Ok(CommandResult::Continue)
}

fn help_command(_command_params: CommandParams) -> io::Result<CommandResult> {
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
    Ok(CommandResult::Continue)
}

fn switch_command(command_params: CommandParams) -> io::Result<CommandResult> {
    let new_history_file = command_params.args.get(0).unwrap_or(&"");

    if new_history_file.is_empty() {
        println!("Error: No history file specified. Usage: :switch <history_file>");
        return Ok(CommandResult::Continue);
    }

    Ok(CommandResult::SwitchHistory(new_history_file.to_string()))
}

fn edit_command(command_params: CommandParams) -> io::Result<CommandResult> {
    let history = command_params.history;
    let editor = env::var("EDITOR")
        .or_else(|_| env::var("VISUAL"))
        .unwrap_or_else(|_| {
            if cfg!(target_os = "windows") {
                "notepad".to_string()
            } else {
                "vi".to_string()
            }
        });

    let status = Command::new(editor).arg(history.path.clone()).status();
    if !status.map_or(false, |s| s.success()) {
        println!("Error opening file in editor");
    }
    history.reload_content();

    Ok(CommandResult::Continue)
}

fn sysprompt_command(command_params: CommandParams) -> io::Result<CommandResult> {
    command_params
        .ollama_client
        .update_system_prompt(command_params.args.join(" "));
    Ok(CommandResult::Continue)
}
