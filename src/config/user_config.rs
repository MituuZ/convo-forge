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
