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

use crate::api::ChatClient;
use crate::command::commands_impl;
use crate::config::profiles_config::ModelType;
use crate::history_file::HistoryFile;
use colored::Colorize;
use std::collections::HashMap;
use std::io;
use std::path::PathBuf;

pub enum CommandResult {
    Continue,
    Quit,
    SwitchHistory(String),
    SwitchContext(Option<PathBuf>),
    HandlePrompt(PathBuf, Option<String>),
    SwitchModel(ModelType),
    PrintModels,
    SwitchProfile(String),
    PrintProfiles,
}

pub struct CommandParams<'a> {
    pub(crate) args: Vec<String>,
    pub(crate) chat_client: &'a mut Box<dyn ChatClient>,
    pub(crate) history: &'a mut HistoryFile,
    pub(crate) cforge_dir: String,
}

impl<'a> CommandParams<'a> {
    pub fn new(
        args: Vec<String>,
        chat_client: &'a mut Box<dyn ChatClient>,
        history: &'a mut HistoryFile,
        cforge_dir: String,
    ) -> Self {
        CommandParams {
            args,
            chat_client,
            history,
            cforge_dir,
        }
    }
}

type CommandFn = fn(CommandParams) -> io::Result<CommandResult>;

pub struct CommandStruct<'a> {
    pub(crate) command_string: &'a str,
    description: &'a str,
    command_example: Option<&'a str>,
    pub(crate) file_command: Option<FileCommandDirectory>,
    pub(crate) command_fn: CommandFn,
    pub(crate) default_prefix: Option<String>,
}

#[derive(Clone, Debug)]
pub enum FileCommandDirectory {
    Knowledge,
    Cforge,
    Prompt,
}

impl<'a> CommandStruct<'a> {
    pub fn new(
        command_string: &'a str,
        description: &'a str,
        command_example: Option<&'a str>,
        file_command: Option<FileCommandDirectory>,
        command_fn: CommandFn,
        default_prefix: Option<String>,
    ) -> Self {
        CommandStruct {
            command_string,
            command_example,
            description,
            file_command,
            command_fn,
            default_prefix,
        }
    }

    pub fn execute(&self, params: CommandParams) -> io::Result<CommandResult> {
        (self.command_fn)(params)
    }

    pub(crate) fn display(&self) -> String {
        match self.command_example {
            Some(example) => format!(
                "{:<12} - {}\n            {}",
                self.command_string.cyan(), self.description, example
            ),
            None => format!("{:<12} - {}", self.command_string.cyan(), self.description),
        }
    }
}

pub(crate) fn create_command_registry<'a>(
    default_prefixes: HashMap<String, String>,
) -> HashMap<String, CommandStruct<'a>> {
    let constructors: Vec<(String, CommandStruct<'a>)> = vec![
        commands_impl::quit::command(&default_prefixes),
        commands_impl::list::command(&default_prefixes),
        commands_impl::switch::command(&default_prefixes),
        commands_impl::help::command(&default_prefixes),
        commands_impl::edit::command(&default_prefixes),
        commands_impl::sysprompt::command(&default_prefixes),
        commands_impl::context::command(&default_prefixes),
        commands_impl::prompt::command(&default_prefixes),
        commands_impl::model::command(&default_prefixes),
        commands_impl::profile::command(&default_prefixes),
        commands_impl::tools::command(&default_prefixes),
    ];

    let mut map: HashMap<String, CommandStruct<'a>> = HashMap::new();
    for (name, cmd) in constructors {
        map.insert(name, cmd);
    }
    map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_command_registry() {
        let temp_map = HashMap::new();
        let registry = create_command_registry(temp_map);

        assert!(registry.contains_key("q"));
        assert!(registry.contains_key("list"));
        assert!(registry.contains_key("switch"));
        assert!(registry.contains_key("sysprompt"));
        assert!(registry.contains_key("help"));
        assert!(registry.contains_key("edit"));
        assert!(registry.contains_key("context"));
        assert!(registry.contains_key("prompt"));
        assert!(registry.contains_key("model"));
        assert!(registry.contains_key("profile"));
        assert!(registry.contains_key("tools"));

        assert_eq!(registry.len(), 11);
    }
}
