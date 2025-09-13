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

use crate::command::command_complete::CommandHelper;
use crate::command::commands::{CommandStruct, FileCommandDirectory};
use crate::config::profiles_config::{Model, Profile};
use crate::config::{cache_config::CacheConfig, rustyline_config::build, user_config::UserConfig};

pub mod cache_config;
pub mod profiles_config;
pub mod rustyline_config;
pub mod user_config;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub cache_config: CacheConfig,
    pub user_config: UserConfig,
    rustyline_config: Config,
    pub data_dir: PathBuf,
    pub prompt_dir: PathBuf,
    pub current_model: Model,
    pub current_profile: Profile,
}

impl AppConfig {
    pub fn load_config() -> AppConfig {
        let mut cache_config: CacheConfig = CacheConfig::load(get_cache_path());
        let user_config: UserConfig = UserConfig::load(get_config_path());
        let rustyline_config = build(&user_config);

        user_config
            .profiles_config
            .validate()
            .expect("Invalid profiles config, see error message above");

        let previous_profile_name = cache_config
            .last_profile_name
            .clone()
            .unwrap_or(user_config.profiles_config.profiles[0].name.clone());

        let initial_profile = user_config.find_profile(&previous_profile_name).clone();

        // Take profile_models out of cache_config and replace it with None to avoid the move issue
        let mut profile_models = cache_config
            .profile_models
            .take()
            .unwrap_or_default()
            .clone();
        let initial_model_type = profile_models.get(&initial_profile.name).cloned();

        let actual_model_type = initial_model_type.unwrap_or(
            initial_profile
                .models
                .first()
                .unwrap_or_else(|| {
                    panic!("No models found for profile '{}'", &initial_profile.name)
                })
                .model_type,
        );

        let initial_model = initial_profile.get_model(&actual_model_type).clone();

        cache_config.last_profile_name = Some(initial_profile.name.clone());
        profile_models.insert(initial_profile.name.clone(), actual_model_type);
        cache_config.profile_models = Some(profile_models);

        AppConfig {
            cache_config,
            user_config,
            rustyline_config,
            data_dir: get_data_path(Some("chats")),
            prompt_dir: get_data_path(Some("prompts")),
            current_model: initial_model,
            current_profile: initial_profile,
        }
    }

    pub fn create_rustyline_editor(
        &self,
        command_registry: &HashMap<String, CommandStruct>,
    ) -> rustyline::Result<Editor<CommandHelper, DefaultHistory>> {
        let command_vecs = get_commands(command_registry);

        let helper = CommandHelper::new(
            command_vecs.all_commands,
            command_vecs.file_commands,
            &self.data_dir.display().to_string(),
            &self.user_config.knowledge_dir,
            &self.prompt_dir.display().to_string(),
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

    pub fn get_profile(&mut self) -> Profile {
        if let Some(last_profile_name) = self.cache_config.last_profile_name.clone() {
            return self.user_config.find_profile(&last_profile_name).clone();
        }

        let profile = self.user_config.profiles_config.profiles[0].clone();
        self.cache_config.last_profile_name = Some(profile.name.clone());
        self.cache_config.save(get_cache_path());
        profile
    }

    pub fn maybe_profile(&mut self, profile_name: &str) -> Option<Profile> {
        for profile in self.user_config.profiles_config.profiles.iter() {
            if profile.name.to_lowercase() == profile_name.to_lowercase() {
                return Some(profile.clone());
            }
        }

        None
    }

    pub fn switch_profile(&mut self, profile: &Profile) {
        self.current_profile = profile.clone();

        self.cache_config.last_profile_name = Some(profile.name.clone());
        self.cache_config.save(get_cache_path());

        let mut profile_models = self.cache_config.profile_models.take().unwrap_or_default();

        if let Some(last_used_model_type) = profile_models.get(&profile.name) {
            self.current_model = profile.get_model(last_used_model_type).clone();
            profile_models.insert(profile.name.clone(), *last_used_model_type);
        } else {
            self.current_model = profile.models[0].clone();
            profile_models.insert(profile.name.clone(), self.current_model.model_type);
        }

        self.cache_config.profile_models = Some(profile_models);
        println!(
            "Switched to profile '{}' and model '{}' ({})",
            profile.name, self.current_model.model, self.current_model.model_type
        );
        self.cache_config.save(get_cache_path());
    }

    pub fn switch_model(&mut self, model: &Model) {
        self.current_model = model.clone();

        let mut profile_models = self.cache_config.profile_models.take().unwrap_or_default();

        profile_models.insert(self.current_profile.name.clone(), model.model_type);
        self.cache_config.profile_models = Some(profile_models);
        self.cache_config.save(get_cache_path());

        println!("Switched to model: {}", model.model);
    }
}

struct CommandVecs {
    all_commands: Vec<(String, Option<String>)>,
    file_commands: Vec<(String, FileCommandDirectory)>,
}

fn get_commands(command_registry: &HashMap<String, CommandStruct>) -> CommandVecs {
    let mut all_commands = Vec::<(String, Option<String>)>::new();
    let mut file_commands = Vec::<(String, FileCommandDirectory)>::new();

    for command in command_registry {
        all_commands.push((
            command.1.command_string.to_string(),
            command.1.default_prefix.clone(),
        ));
        if let Some(file_command) = command.1.file_command.as_ref() {
            file_commands.push((command.1.command_string.to_string(), file_command.clone()));
        }
    }

    CommandVecs { all_commands, file_commands }
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
fn get_data_path(additional_path: Option<&str>) -> PathBuf {
    let data_path = match additional_path {
        None => dirs_next::data_dir()
            .expect("Could not determine data directory location")
            .join("cforge"),
        Some(additional_path) => dirs_next::data_dir()
            .expect("Could not determine data directory location")
            .join("cforge")
            .join(additional_path),
    };

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

    use crate::command::commands::{
        CommandParams, CommandResult, CommandStruct, FileCommandDirectory,
    };
    use crate::config::get_commands;

    #[test]
    fn get_commands_base() {
        let command_registry = create_registry();

        let command_vecs = get_commands(&command_registry);
        assert_eq!(3, command_vecs.all_commands.len());
        assert_eq!(2, command_vecs.file_commands.len());
    }

    #[test]
    fn get_commands_empty_registry() {
        let command_registry: HashMap<String, CommandStruct> = HashMap::new();

        let command_vecs = get_commands(&command_registry);
        assert_eq!(0, command_vecs.all_commands.len());
        assert_eq!(0, command_vecs.file_commands.len());
    }

    fn create_registry<'a>() -> HashMap<String, CommandStruct<'a>> {
        let mut command_registry: HashMap<String, CommandStruct> = HashMap::new();

        let command1 = CommandStruct::new("cmd1", "", None, None, nop, None);
        let command2 = CommandStruct::new(
            "cmd2",
            "",
            None,
            Some(FileCommandDirectory::Cforge),
            nop,
            None,
        );
        let command3 = CommandStruct::new(
            "cmd3",
            "",
            None,
            Some(FileCommandDirectory::Knowledge),
            nop,
            None,
        );

        command_registry.insert("cmd1".to_string(), command1);
        command_registry.insert("cmd2".to_string(), command2);
        command_registry.insert("cmd3".to_string(), command3);

        command_registry
    }

    fn nop(_: CommandParams) -> Result<CommandResult> {
        Ok(CommandResult::Continue)
    }
}
