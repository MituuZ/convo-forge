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

use crate::config::user_config::UserConfig;

#[derive(Debug, Deserialize, Default, Serialize)]
pub struct RustylineConfig {
    #[serde(default)]
    pub edit_mode: EditMode,

    #[serde(default)]
    pub completion_type: CompletionType,
}

pub(crate) fn build(user_config: &UserConfig) -> rustyline::Config {
    let mut config_builder = rustyline::Config::builder();

    // Apply edit mode setting
    config_builder = match user_config.rustyline.edit_mode {
        EditMode::Emacs => config_builder.edit_mode(rustyline::EditMode::Emacs),
        EditMode::Vi => config_builder.edit_mode(rustyline::EditMode::Vi),
    };

    config_builder = match user_config.rustyline.completion_type {
        CompletionType::Circular => {
            config_builder.completion_type(rustyline::CompletionType::Circular)
        }
        CompletionType::List => config_builder.completion_type(rustyline::CompletionType::List),
    };

    config_builder.build()
}

#[derive(Debug, Clone, Copy, Deserialize, Default, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum EditMode {
    #[default]
    Emacs,
    Vi,
}

#[derive(Debug, Clone, Copy, Deserialize, Default, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CompletionType {
    #[default]
    Circular,
    List,
}
