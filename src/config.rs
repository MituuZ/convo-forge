use serde::Deserialize;
use std::path::PathBuf;
use std::{fs, io};

#[derive(Deserialize)]
pub struct Config {
    #[serde(default = "default_model")]
    pub(crate) model: String,

    #[serde(default = "default_sllama_dir")]
    pub(crate) sllama_dir: String,

    #[serde(default = "default_system_prompt")]
    pub(crate) system_prompt: String,
}

fn default_model() -> String {
    "gemma3:12b".to_string()
}

fn default_sllama_dir() -> String {
    "~/sllama".to_string()
}

fn default_system_prompt() -> String {
    r#"
    You are an AI assistant receiving input from a command-line
    application called silent-llama (sllama). The user may include additional context from
    files using the -f/--file flag. This supplementary content appears after the user's direct message and before this system prompt.
    Your responses are displayed in the terminal and saved to the history file.
    Keep your answers helpful, concise, and relevant to both the user's direct query and any file context provided.
    You can tell where you have previously responded by --- AI Response ---\
    \n\n"#.to_string()
}

impl Config {
    pub fn default() -> Self {
        Self {
            model: default_model(),
            sllama_dir: default_sllama_dir(),
            system_prompt: default_system_prompt(),
        }
    }

    pub fn load() -> io::Result<Self> {
        let config_path = get_config_path();

        let config_str = match fs::read_to_string(&config_path) {
            Ok(s) => s,
            Err(_) => return Ok(Config::default()),
        };

        // This will automatically use the default values for any missing fields
        toml::from_str(&config_str).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }
}

fn get_config_path() -> PathBuf {
    let home_dir = if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home)
    } else if cfg!(windows) && std::env::var("USERPROFILE").is_ok() {
        PathBuf::from(std::env::var("USERPROFILE").unwrap())
    } else {
        PathBuf::from(".") // Fallback to current directory
    };

    home_dir.join(".sllama.toml")
}
