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
use std::{collections::HashMap, fs::create_dir_all, path::PathBuf};

use rustyline::{history::DefaultHistory, Cmd, Config, Editor, EventHandler, KeyEvent, Modifiers};

use crate::{
    command_complete::CommandHelper,
    commands::{CommandStruct, FileCommand},
    config::{cache_config::CacheConfig, rustyline_config::build, user_config::UserConfig},
};

pub mod cache_config;
pub mod rustyline_config;
pub mod user_config;

pub struct AppConfig {
    pub cache_config: CacheConfig,
    pub user_config: UserConfig,
    rustyline_config: Config,
    pub data_dir: PathBuf,
}

impl AppConfig {
    pub fn load_config() -> AppConfig {
        let cache_config: CacheConfig = CacheConfig::load(get_cache_path());
        let user_config: UserConfig = UserConfig::load(get_config_path());
        let rustyline_config = build(&user_config);

        AppConfig {
            cache_config,
            user_config,
            rustyline_config,
            data_dir: get_data_path(),
        }
    }

    pub fn create_rustyline_editor(
        &self,
        command_registry: &HashMap<String, CommandStruct>,
    ) -> rustyline::Result<Editor<CommandHelper, DefaultHistory>> {
        let (commands, file_commands) = get_commands(command_registry);

        let helper = CommandHelper::new(
            commands,
            file_commands,
            &self.data_dir.display().to_string(),
            &self.user_config.knowledge_dir,
        );
        let mut editor = Editor::with_config(self.rustyline_config)?;
        editor.set_helper(Some(helper));

        editor.bind_sequence(
            KeyEvent(rustyline::KeyCode::Enter, Modifiers::ALT),
            EventHandler::Simple(Cmd::Newline),
        );

        Ok(editor)
    }

    pub fn update_last_history_file(&mut self, history_file: String) {
        self.cache_config.last_history_file = Some(history_file);
        self.cache_config.save(get_cache_path());
    }
}

fn get_commands(
    command_registry: &HashMap<String, CommandStruct>,
) -> (Vec<(String, Option<String>)>, Vec<(String, FileCommand)>) {
    let mut all_commands = Vec::<(String, Option<String>)>::new();
    let mut file_commands = Vec::<(String, FileCommand)>::new();

    for command in command_registry {
        all_commands.push((command.1.command_string.to_string(), command.1.default_prefix.clone()));
        if let Some(file_command) = command.1.file_command.as_ref() {
            file_commands.push((command.1.command_string.to_string(), file_command.clone()));
        }
    }

    (all_commands, file_commands)
}

/// Return XDG compliant config path
/// e.g. `~/.config/cforge`
///
/// Returns a `PathBuf` or panics if config cannot be determined
fn get_config_path() -> PathBuf {
    let config_path = dirs_next::config_dir()
        .expect("Could not determine config directory location")
        .join("cforge");

    init_dir(config_path)
}

/// Return XDG compliant data path
/// e.g. `~/.local/share/cforge`
///
/// Returns a `PathBuf` or panics if data path cannot determined
fn get_data_path() -> PathBuf {
    let data_path = dirs_next::data_dir()
        .expect("Could not determine data directory location")
        .join("cforge");

    init_dir(data_path)
}

fn init_dir(path: PathBuf) -> PathBuf {
    create_dir_all(&path).unwrap_or_else(|e| {
        panic!("Failed to create data directory at {}: {e}", path.display());
    });

    path
}

/// Return XDG compliant cache path
/// e.g. `~/.local/share/cforge`
///
/// Cache is not explicitly required so we can return an `Option`
fn get_cache_path() -> Option<PathBuf> {
    if let Some(cache_dir) = dirs_next::cache_dir() {
        let cache_path = cache_dir.join("cforge");
        if let Err(e) = create_dir_all(&cache_path) {
            eprintln!("Failed to create cache directory: {e}");
            return None;
        }

        return Some(cache_path);
    }

    eprintln!("Could not determine cache directory");
    None
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, io::Result};

    use crate::{
        commands::{CommandParams, CommandResult, CommandStruct, FileCommand},
        config::get_commands,
    };

    #[test]
    fn get_commands_base() {
        let command_registry = create_registry();

        let (commands, file_commands) = get_commands(&command_registry);
        assert_eq!(3, commands.len());
        assert_eq!(2, file_commands.len());
    }

    #[test]
    fn get_commands_empty_registry() {
        let command_registry: HashMap<String, CommandStruct> = HashMap::new();

        let (commands, file_commands) = get_commands(&command_registry);
        assert_eq!(0, commands.len());
        assert_eq!(0, file_commands.len());
    }

    fn create_registry<'a>() -> HashMap<String, CommandStruct<'a>> {
        let mut command_registry: HashMap<String, CommandStruct> = HashMap::new();

        let command1 = CommandStruct::new("cmd1", "", None, None, nop, None);
        let command2 = CommandStruct::new("cmd2", "", None, Some(FileCommand::CforgeDir), nop, None);
        let command3 = CommandStruct::new("cmd3", "", None, Some(FileCommand::KnowledgeDir), nop, None);

        command_registry.insert("cmd1".to_string(), command1);
        command_registry.insert("cmd2".to_string(), command2);
        command_registry.insert("cmd3".to_string(), command3);

        command_registry
    }

    fn nop(_: CommandParams) -> Result<CommandResult> {
        Ok(CommandResult::Continue)
    }
}
