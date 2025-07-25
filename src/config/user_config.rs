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
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

use crate::config::rustyline_config::RustylineConfig;

const CONFIG_FILE: &str = "cforge.toml";

#[derive(Deserialize, Serialize)]
pub struct UserConfig {
    #[serde(default = "default_model")]
    pub model: String,

    #[serde(default = "default_knowledge_dir")]
    pub knowledge_dir: String,

    #[serde(default = "default_system_prompt")]
    pub system_prompt: String,

    #[serde(default)]
    pub rustyline: RustylineConfig,

    #[serde(default = "default_token_estimation")]
    pub token_estimation: bool,

    #[serde(default = "default_provider")]
    pub provider: String,

    #[serde(default = "default_max_tokens")]
    pub max_tokens: usize,
}

impl UserConfig {
    pub fn load(config_path: PathBuf) -> Self {
        let config_str = fs::read_to_string(config_path.join(CONFIG_FILE)).unwrap_or_else(|e| {
            panic!("Could not read config file: {e}");
        });

        let config: UserConfig = toml::from_str(&config_str).unwrap_or_else(|e| {
            panic!("Could not parse config toml: {e}");
        });

        config
    }
}

impl Default for UserConfig {
    fn default() -> Self {
        Self {
            model: default_model(),
            knowledge_dir: default_knowledge_dir(),
            system_prompt: default_system_prompt(),
            rustyline: RustylineConfig::default(),
            token_estimation: default_token_estimation(),
            provider: default_provider(),
            max_tokens: default_max_tokens(),
        }
    }
}

fn default_model() -> String {
    "gemma3:12b".to_string()
}

fn default_token_estimation() -> bool {
    true
}

fn default_provider() -> String {
    "ollama".to_string()
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
    application called convo-forge (cforge). The user may include additional context from another file,
    this is included as a separate user prompt.
    Your responses are displayed in the terminal and saved to the history file.
    Keep your answers helpful, concise, and relevant to both the user's direct query and any file context provided.
    \n\n"#.to_string()
}

#[cfg(test)]
mod tests {
    use std::{fs::write, path::PathBuf};

    use tempfile::TempDir;

    use crate::config::user_config::{CONFIG_FILE, UserConfig};

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
        assert_eq!("anthropic", config.provider);
    }

    #[test]
    fn load_empty_config_file() {
        let temp_dir = create_config("");
        let config = UserConfig::load(temp_dir.path().to_path_buf());

        // Should use defaults
        assert_eq!(true, config.token_estimation);
        assert_eq!("ollama", config.provider);
        assert_eq!("gemma3:12b", config.model);
        assert_eq!(1024, config.max_tokens);
        assert_eq!("", config.knowledge_dir);
    }

    fn create_config(content: &str) -> TempDir {
        let temp_dir: TempDir = TempDir::new().unwrap();
        let config_path: PathBuf = temp_dir.path().join(CONFIG_FILE);
        write(&config_path, content).expect("Writing to test config failed");
        temp_dir
    }
}
