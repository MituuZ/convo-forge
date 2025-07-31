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
use std::collections::HashMap;
use std::io;
use std::path::PathBuf;

use crate::api::ChatApi;
use crate::command::commands::{CommandParams, CommandResult, CommandStruct};
use crate::config::AppConfig;
use crate::history_file::HistoryFile;
use crate::user_input::{Command, UserInput};

pub(crate) struct CommandProcessor<'a> {
    chat_api: &'a mut Box<dyn ChatApi>,
    history: &'a mut HistoryFile,
    app_config: &'a mut AppConfig,
    command_registry: &'a HashMap<String, CommandStruct<'a>>,
    context_file_path: &'a mut Option<PathBuf>,
    context_file_content: Option<String>,
}

impl<'a> CommandProcessor<'a> {
    pub fn new(
        chat_api: &'a mut Box<dyn ChatApi>,
        history: &'a mut HistoryFile,
        app_config: &'a mut AppConfig,
        command_registry: &'a HashMap<String, CommandStruct<'a>>,
        context_file_path: &'a mut Option<PathBuf>,
        context_file_content: Option<String>,
    ) -> Self {
        Self {
            chat_api,
            history,
            app_config,
            command_registry,
            context_file_path,
            context_file_content,
        }
    }

    pub fn process(&mut self, input: &str) -> io::Result<CommandResult> {
        match UserInput::parse(input) {
            UserInput::Command(command) => self.handle_command(command),
            UserInput::Prompt(prompt) => self.handle_prompt(prompt),
        }
    }

    fn handle_command(&mut self, command: Command) -> io::Result<CommandResult> {
        let command_params = CommandParams::new(
            command.args,
            self.chat_api,
            self.history,
            self.app_config.data_dir.display().to_string(),
        );

        if let Some(cmd) = self.command_registry.get(&command.name) {
            let result = cmd.execute(command_params)?;

            if let CommandResult::SwitchHistory(new_file) = &result {
                *self.history = HistoryFile::new(
                    new_file.clone(),
                    self.app_config.data_dir.display().to_string(),
                )?;
                self.app_config.update_last_history_file(new_file.clone());
                println!("{}", self.history.get_content());
                println!("Switched to history file: {}", self.history.filename);
            }

            if let CommandResult::SwitchContext(new_context) = &result {
                match new_context {
                    Some(new_path) => {
                        *self.context_file_path = Some(new_path.clone());
                        println!("Updated context file");
                    }
                    None => {
                        *self.context_file_path = None;
                        println!("Removed context file");
                    }
                }
            }

            Ok(result)
        } else {
            println!("Unknown command: {}", command.name);
            Ok(CommandResult::Continue)
        }
    }

    fn handle_prompt(&mut self, prompt: String) -> io::Result<CommandResult> {
        let history_json = match self.history.get_content_json() {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Error reading history file: {e}");
                return Ok(CommandResult::Quit);
            }
        };

        let llm_response = self.chat_api.generate_response(
            history_json,
            &prompt,
            self.context_file_content.as_deref(),
        )?;

        self.history.append_user_input(&prompt)?;

        // Print the AI response with the delimiter to make it easier to parse
        println!("{}", self.history.append_ai_response(&llm_response)?);

        Ok(CommandResult::Continue)
    }
}
