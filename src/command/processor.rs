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
use std::path::PathBuf;
use std::{fs, io};

use crate::api::ChatApi;
use crate::command::command_util::get_editor;
use crate::command::commands::{CommandParams, CommandResult, CommandStruct};
use crate::config::profiles_config::{ModelType, Profile};
use crate::config::AppConfig;
use crate::history_file::HistoryFile;
use crate::user_input::{Command, UserInput};

pub(crate) struct CommandProcessor<'a> {
    chat_api: &'a mut Box<dyn ChatApi>,
    history: &'a mut HistoryFile,
    app_config: &'a mut AppConfig,
    command_registry: &'a HashMap<String, CommandStruct<'a>>,
    context_file_path: &'a mut Option<PathBuf>,
    update_chat_api: &'a mut bool,
    current_profile: &'a mut Profile,
    current_model_type: &'a mut ModelType,
    context_file_content: Option<String>,
}

// TODO: Use this
pub struct ApiConfig<'a> {
    update_chat_api: &'a mut bool,
    model_type: &'a mut String,
    profile: &'a mut String,
}

impl<'a> CommandProcessor<'a> {
    pub fn new(
        chat_api: &'a mut Box<dyn ChatApi>,
        history: &'a mut HistoryFile,
        app_config: &'a mut AppConfig,
        command_registry: &'a HashMap<String, CommandStruct<'a>>,
        context_file_path: &'a mut Option<PathBuf>,
        update_chat_api: &'a mut bool,
        current_profile: &'a mut Profile,
        current_model_type: &'a mut ModelType,
        context_file_content: Option<String>,
    ) -> Self {
        Self {
            chat_api,
            history,
            app_config,
            command_registry,
            context_file_path,
            update_chat_api,
            current_profile,
            current_model_type,
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

            match &result {
                CommandResult::SwitchHistory(new_file) => {
                    *self.history = HistoryFile::new(
                        new_file.clone(),
                        self.app_config.data_dir.display().to_string(),
                    )?;
                    self.app_config.update_last_history_file(new_file.clone());
                    println!("{}", self.history.get_content());
                    println!("Switched to history file: {}", self.history.filename);
                }
                CommandResult::SwitchContext(new_context) => match new_context {
                    Some(new_path) => {
                        *self.context_file_path = Some(new_path.clone());
                        println!("Updated context file");
                    }
                    None => {
                        *self.context_file_path = None;
                        println!("Removed context file");
                    }
                },
                CommandResult::HandlePrompt(prompt_file, user_prompt) => {
                    match user_prompt {
                        None => {
                            let editor = get_editor();

                            let status = std::process::Command::new(editor).arg(prompt_file).status();
                            if !status.is_ok_and(|s| s.success()) {
                                eprintln!("Error opening file in editor");
                            }
                        }
                        Some(user_prompt) => {
                            let combined_prompt = Self::combine(prompt_file, user_prompt);
                            self.handle_prompt(combined_prompt)?;
                        }
                    }
                }
                CommandResult::SwitchModel(new_model) => {
                    self.app_config.switch_model_type(new_model);
                    // self.app_config.switch_model_type(&new_model);
                    *self.update_chat_api = true;
                    // *self.current_model_type = new_model.clone();
                    println!("Switched to model: {}", self.current_profile.get_model(new_model).model);
                }
                CommandResult::SwitchProfile(new_profile) => {
                    self.app_config.switch_profile(&new_profile);
                    *self.update_chat_api = true;
                    // self.app_config.cache_config.last_profile_name = Some(new_profile.clone());
                    // *self.current_profile = self.app_config.get_profile();
                    // println!("Switched to profile: {}", new_profile);
                }
                CommandResult::PrintModels => {
                    let current_profile = self.app_config.get_profile();
                    println!("Available models for profile {}:", current_profile.name);
                    for model in current_profile.models {
                        if model == self.app_config.current_model {
                            println!("* Model: {}, type: {}", model.model, model.model_type);
                        } else {
                            println!("Model: {}, type: {}", model.model, model.model_type);
                        }
                    }
                }
                CommandResult::PrintProfiles => {
                    for profile in &self.app_config.user_config.profiles_config.profiles {
                        if *profile == self.app_config.current_profile {
                            println!("\n* Profile: {}", profile.name);
                        } else {
                            println!("\nProfile: {}", profile.name);
                        }
                        println!("Provider: {}", profile.provider);
                        println!("Models:");
                        println!("-------");
                        for model in &profile.models {
                            if let Some(desc) = &model.description {
                                println!("  - {}\n    Type: {}\n    Description: {}",
                                         model.model, model.model_type, desc);
                            } else {
                                println!("  - {}\n    Type: {}",
                                         model.model, model.model_type);
                            }
                        }
                    }
                }
                _ => {}
            }

            Ok(result)
        } else {
            println!("Unknown command: {}", command.name);
            Ok(CommandResult::Continue)
        }
    }

    fn combine(prompt_file: &PathBuf, user_prompt: &str) -> String {
        let prompt_content = fs::read_to_string(prompt_file).unwrap_or_else(|_| String::new());

        if prompt_content.contains("${{user_prompt}}") {
            prompt_content.replace("${{user_prompt}}", user_prompt)
        } else {
            format!("{prompt_content}{user_prompt}")
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
