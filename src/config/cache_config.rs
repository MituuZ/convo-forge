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

use std::{
    fs::{read_to_string, write},
    io,
    path::PathBuf,
};

use serde::{Deserialize, Serialize};

const CACHE_FILE: &str = "cforge.cache.toml";

#[derive(Deserialize, Serialize)]
pub struct CacheConfig {
    pub last_history_file: Option<String>,
}

impl CacheConfig {
    fn new(last_history_file: Option<String>) -> Self {
        Self { last_history_file }
    }

    fn empty() -> Self {
        Self::new(None)
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
