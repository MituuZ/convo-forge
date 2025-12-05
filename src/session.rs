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
use crate::api::get_chat_client_implementation;
use crate::config::AppConfig;
use crate::models::history_file::HistoryFile;

pub(crate) struct Session {
    pub(crate) config: AppConfig,
    pub(crate) client: Box<dyn crate::api::ChatClient>,
    pub(crate) history_file: HistoryFile,
}

impl Session {
    pub(crate) fn new(config: AppConfig) -> Self {
        let client = Self::create_client(&config);
        let history_file = HistoryFile::new(
            config.cache_config.get_history_file_path(),
            config.data_dir.display().to_string(),
        ).expect("Failed to determine history file");

        Self {
            config,
            client,
            history_file,
        }
    }

    fn create_client(config: &AppConfig) -> Box<dyn crate::api::ChatClient> {
        get_chat_client_implementation(
            &config.current_profile.provider,
            &config.current_model.model,
            config.user_config.system_prompt.clone(),
            config.user_config.max_tokens,
        )
    }

    pub(crate) fn rebuild_client(&mut self) {
        self.client = Self::create_client(&self.config);
    }

    pub fn update_config<F>(&mut self, modifier: F)
    where
        F: FnOnce(&mut AppConfig) -> bool,
    {
        let should_rebuild = modifier(&mut self.config);
        if should_rebuild {
            self.rebuild_client();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Session;
    use crate::config::profiles_config::{Model, ModelType, Profile};
    use crate::config::{AppConfig, CacheConfig, UserConfig};
    use tempfile::TempDir;

    fn make_test_config(temp: &TempDir, history_name: &str, system_prompt: &str) -> AppConfig {
        let mut config = AppConfig::default();

        // point data_dir to a temporary location to avoid touching user directories
        let chats_dir = temp.path().join("chats");
        std::fs::create_dir_all(&chats_dir).unwrap();
        config.data_dir = chats_dir;

        // minimal cache config so HistoryFile path is resolvable
        config.cache_config = CacheConfig {
            last_history_file: Some(history_name.to_string()),
            last_profile_name: Some("test-profile".to_string()),
            profile_models: None,
        };

        // minimal user config
        let mut user = UserConfig::default();
        user.system_prompt = system_prompt.to_string();
        config.user_config = user.clone();

        // set a test profile that maps to our test stub ChatClient
        config.current_profile = Profile {
            name: "test-profile".to_string(),
            provider: "test".to_string(),
            models: vec![Model {
                model: "test-model".to_string(),
                description: Some("test model".to_string()),
                model_type: ModelType::Fast,
            }],
        };

        config.current_model = Model {
            model: "test-model".to_string(),
            description: Some("test model".to_string()),
            model_type: ModelType::Fast,
        };

        config
    }

    #[test]
    fn session_new_initializes_history_file() {
        let temp = TempDir::new().unwrap();
        let config = make_test_config(&temp, "sess.hist", "sp-1");

        let session = Session::new(config);

        // History file filename should match
        assert_eq!(session.history_file.filename, "sess.hist");
        // File should exist on disk
        assert!(std::path::Path::new(&session.history_file.path).exists());
    }

    #[test]
    fn update_config_without_rebuild_keeps_client() {
        let temp = TempDir::new().unwrap();
        let mut session = Session::new(make_test_config(&temp, "sess2.hist", "sp-initial"));

        // Changing the system prompt but returning false should not rebuild client
        session.update_config(|c| {
            c.user_config.system_prompt = "sp-updated".to_string();
            false
        });

        assert_eq!(session.client.system_prompt(), "sp-initial".to_string());
    }

    #[test]
    fn update_config_with_rebuild_updates_client() {
        let temp = TempDir::new().unwrap();
        let mut session = Session::new(make_test_config(&temp, "sess3.hist", "sp-a"));

        session.update_config(|c| {
            c.user_config.system_prompt = "sp-b".to_string();
            true // request rebuild
        });

        assert_eq!(session.client.system_prompt(), "sp-b".to_string());
    }

    #[test]
    fn rebuild_client_applies_current_config() {
        let temp = TempDir::new().unwrap();
        let mut session = Session::new(make_test_config(&temp, "sess4.hist", "sp-0"));

        session.config.user_config.system_prompt = "sp-1".to_string();
        session.rebuild_client();

        assert_eq!(session.client.system_prompt(), "sp-1".to_string());
    }
}