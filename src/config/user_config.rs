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
use crate::config::profiles_config::{Profile, ProfilesConfig};
use crate::config::rustyline_config::RustylineConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::{fs, path::PathBuf};

const CONFIG_FILE: &str = "cforge.toml";

#[derive(Deserialize, Serialize)]
pub struct UserConfig {
    #[serde(default = "default_knowledge_dir")]
    pub knowledge_dir: String,

    #[serde(default = "default_system_prompt")]
    pub system_prompt: String,

    #[serde(default = "default_token_estimation")]
    pub token_estimation: bool,

    #[serde(default = "default_max_tokens")]
    pub max_tokens: usize,

    #[serde(default = "default_command_prefixes")]
    pub command_prefixes: HashMap<String, String>,

    #[serde(default)]
    pub rustyline: RustylineConfig,

    #[serde(default)]
    pub profiles_config: ProfilesConfig,
}

impl UserConfig {
    pub fn load(config_path: PathBuf) -> Self {
        let path = config_path.join(CONFIG_FILE);

        if !path.exists() {
            let default = UserConfig::default();
            let toml_str =
                toml::to_string_pretty(&default).expect("Could not serialize default config");
            fs::write(&path, toml_str).expect("Could not write default config file");
            println!("Created default config at {:?}", path);
            return default;
        }

        let config_str = fs::read_to_string(&path).unwrap_or_else(|e| {
            panic!("Could not read config file: {e}");
        });

        let config: UserConfig = toml::from_str(&config_str).unwrap_or_else(|e| {
            panic!("Could not parse config toml: {e}");
        });

        config
    }

    /// This method searches for a profile with the given `profile_name`
    /// in the list of profiles. If a profile with the specified name is
    /// found, it is returned. Otherwise, the first profile in the list is
    /// returned as a fallback. The fallback behavior assumes that the
    /// `profiles_config` is never empty because it has been validated
    /// during the `load()` process.
    ///
    /// # Returns
    ///
    /// * A reference to the `Profile` that matches the given name, or the
    ///   first profile in the list if no match is found.
    ///
    /// # Panics
    ///
    /// This function will panic if it attempts to unwrap the first profile
    /// and `profiles_config.profiles` is empty. However, this situation
    /// should not occur because `profiles_config` is assumed to be validated
    /// during the `load()` process to ensure that it always contains at least
    /// one profile.
    pub fn find_profile(&self, profile_name: &str) -> Profile {
        match self
            .profiles_config
            .profiles
            .iter()
            .find(|profile| profile.name == profile_name)
        {
            Some(profile) => profile.clone(),
            // This can never be empty, because profiles_config is validated in load()
            None => {
                let profile = self.profiles_config.profiles.first().unwrap();
                eprintln!(
                    "Profile {} not found, using {} profile instead",
                    profile_name, profile.name
                );
                profile.clone()
            }
        }
    }
}

impl Default for UserConfig {
    fn default() -> Self {
        Self {
            knowledge_dir: default_knowledge_dir(),
            system_prompt: default_system_prompt(),
            rustyline: RustylineConfig::default(),
            token_estimation: default_token_estimation(),
            max_tokens: default_max_tokens(),
            command_prefixes: default_command_prefixes(),
            profiles_config: ProfilesConfig::default(),
        }
    }
}

fn default_command_prefixes() -> HashMap<String, String> {
    let mut path_aliases: HashMap<String, String> = HashMap::new();

    path_aliases.insert("switch".into(), "@c/".into());
    path_aliases.insert("list".into(), "@c/".into());
    path_aliases.insert("context".into(), "@k/".into());
    path_aliases.insert("prompt".into(), "@p/".into());

    path_aliases
}

fn default_token_estimation() -> bool {
    true
}

fn default_max_tokens() -> usize {
    1024
}

fn default_knowledge_dir() -> String {
    "".to_string()
}

fn default_system_prompt() -> String {
    r#"
    You are an AI assistant receiving input from a command-line
    application called convo-forge (cforge). The user may include additional context from another file.
    Your responses are displayed in the terminal and saved to the history file.
    Keep your answers helpful, concise, and relevant to both the user's direct query and any file context provided.
    \n\n"#.to_string()
}

#[cfg(test)]
mod tests {
    use std::{fs::write, path::PathBuf};

    use crate::config::rustyline_config::RustylineConfig;
    use crate::config::user_config::{UserConfig, CONFIG_FILE};
    use tempfile::TempDir;

    #[test]
    fn default_values() {
        let config = UserConfig::default();
        assert_eq!(true, config.token_estimation);
        assert_eq!(1024, config.max_tokens);
        assert_eq!("", config.knowledge_dir);

        assert_eq!(
            r#"
    You are an AI assistant receiving input from a command-line
    application called convo-forge (cforge). The user may include additional context from another file.
    Your responses are displayed in the terminal and saved to the history file.
    Keep your answers helpful, concise, and relevant to both the user's direct query and any file context provided.
    \n\n"#,
            config.system_prompt
        );

        assert_eq!("@c/", config.command_prefixes.get("switch").unwrap());
        assert_eq!("@c/", config.command_prefixes.get("list").unwrap());
        assert_eq!("@k/", config.command_prefixes.get("context").unwrap());
        assert_eq!("@p/", config.command_prefixes.get("prompt").unwrap());
        assert_eq!(4, config.command_prefixes.len());

        assert_eq!(RustylineConfig::default(), config.rustyline);
    }

    #[test]
    #[should_panic]
    fn load_invalid_config_file() {
        let temp_dir = create_config(
            "
            thisisa malformed \" string !\"#¤
            ",
        );
        UserConfig::load(temp_dir.path().to_path_buf());
    }

    #[test]
    #[should_panic]
    fn load_non_existent_config_file() {
        let temp_dir = create_config("");
        UserConfig::load(temp_dir.path().join("doesnt_exist.toml").to_path_buf());
    }

    #[test]
    fn load_valid_config_file() {
        let temp_dir = create_config(
            "
            token_estimation = false
            provider = \"anthropic\"
            ",
        );
        let config = UserConfig::load(temp_dir.path().to_path_buf());

        assert_eq!(false, config.token_estimation);
    }

    #[test]
    fn load_empty_config_file() {
        let temp_dir = create_config("");
        let config = UserConfig::load(temp_dir.path().to_path_buf());

        // Should use defaults
        assert_eq!(true, config.token_estimation);
        assert_eq!(1024, config.max_tokens);
        assert_eq!("", config.knowledge_dir);
    }

    #[test]
    fn test_prefixes() {
        let temp_dir = create_config(
            r#"
            [command_prefixes]
            switch = "never"
            list = "gonna"
            context = "give"
            "#,
        );
        let config = UserConfig::load(temp_dir.path().to_path_buf());

        assert_eq!("never", config.command_prefixes.get("switch").unwrap());
        assert_eq!("gonna", config.command_prefixes.get("list").unwrap());
        assert_eq!("give", config.command_prefixes.get("context").unwrap());
    }

    fn create_config(content: &str) -> TempDir {
        let temp_dir: TempDir = TempDir::new().unwrap();
        let config_path: PathBuf = temp_dir.path().join(CONFIG_FILE);
        write(&config_path, content).expect("Writing to test config failed");
        temp_dir
    }
}
