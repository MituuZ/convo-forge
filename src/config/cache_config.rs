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

use crate::config::profiles_config::ModelType;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::{
    fs::{read_to_string, write},
    io,
    path::PathBuf,
};

const CACHE_FILE: &str = "cforge.cache.toml";

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CacheConfig {
    pub last_history_file: Option<String>,
    pub last_profile_name: Option<String>,
    pub profile_models: Option<HashMap<String, ModelType>>,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self::empty()
    }
}

impl CacheConfig {
    fn new(
        last_history_file: Option<String>,
        last_profile_name: Option<String>,
        profile_models: Option<HashMap<String, ModelType>>,
    ) -> Self {
        Self {
            last_history_file,
            last_profile_name,
            profile_models,
        }
    }

    fn empty() -> Self {
        Self::new(None, None, None)
    }

    pub(crate) fn load(cache_path: Option<PathBuf>) -> Self {
        let mut cache = Self::empty();

        if let Some(cache_path) = cache_path {
            match read_to_string(cache_path.join(CACHE_FILE)) {
                Ok(cache_string) => match toml::from_str(&cache_string) {
                    Ok(res) => cache = res,
                    Err(e) => eprintln!("Failed to parse cache toml: {e}"),
                },
                Err(e) => eprintln!("Failed to read cache file: {e}"),
            }
        };

        cache
    }

    pub fn save(&self, cache_path: Option<PathBuf>) {
        if let Some(cache_path) = cache_path {
            match toml::to_string(&self).map_err(io::Error::other) {
                Ok(config_str) => {
                    if let Err(e) = write(cache_path.join(CACHE_FILE), config_str) {
                        eprintln!("Failed to write cache file: {e}");
                    }
                }
                Err(e) => eprintln!("Failed to parse cache config: {e}"),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{fs::write, path::PathBuf};

    use tempfile::TempDir;

    use crate::config::cache_config::{CacheConfig, CACHE_FILE};

    #[test]
    fn load_invalid_cache_config() {
        let temp_dir = create_cache_config(
            "
            thisisa malformed \" string !\"#¤
            ",
        );
        let path_opt = Some(temp_dir.path().to_path_buf());
        let config = CacheConfig::load(path_opt);

        assert_eq!(
            config.last_history_file,
            CacheConfig::empty().last_history_file
        );
    }

    #[test]
    fn load_non_existent_cache_config() {
        let temp_dir = create_cache_config("");
        let path_opt = Some(temp_dir.path().join("doesnt_exist.toml").to_path_buf());
        let config = CacheConfig::load(path_opt);

        assert_eq!(
            config.last_history_file,
            CacheConfig::empty().last_history_file
        );
    }

    #[test]
    fn load_empty_cache_config() {
        let temp_dir = create_cache_config("");
        let path_opt = Some(temp_dir.path().to_path_buf());
        let config = CacheConfig::load(path_opt);

        assert_eq!(
            config.last_history_file,
            CacheConfig::empty().last_history_file
        );
    }

    #[test]
    fn load_valid_cache_config() {
        let temp_dir = create_cache_config("last_history_file = \"some_history_file\"");
        let path_opt = Some(temp_dir.path().to_path_buf());
        let config = CacheConfig::load(path_opt);

        assert_eq!(
            config.last_history_file,
            Some("some_history_file".to_string())
        );
    }

    fn create_cache_config(content: &str) -> TempDir {
        let temp_dir: TempDir = TempDir::new().unwrap();
        let config_path: PathBuf = temp_dir.path().join(CACHE_FILE);
        write(&config_path, content).expect("Writing to test config failed");
        temp_dir
    }
}
