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

use crate::command_complete::CommandHelper;
use crate::commands::{CommandStruct, FileCommand};
use rustyline::history::DefaultHistory;
use rustyline::{Cmd, Editor, EventHandler, KeyEvent, Modifiers};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::{fs, io};

#[cfg(test)]
mod tests {
    use crate::config::{EditMode, UserConfig};

    #[test]
    fn test_default_config_values() {
        let config = UserConfig::default();

        // Check default values directly
        assert_eq!(config.model, "gemma3:12b");
        assert!(config.system_prompt.contains("You are an AI assistant"));

        // For cforge_dir, just check that it ends with "/cforge" or "\cforge"
        // rather than testing the specific home directory path
        assert!(config.cforge_dir.ends_with("/cforge") || config.cforge_dir.ends_with("\\cforge"));

        // Check rustyline default values
        matches!(config.rustyline.edit_mode, EditMode::Emacs);
        matches!(
            config.rustyline.completion_type,
            crate::config::CompletionType::Circular
        );
    }
}
