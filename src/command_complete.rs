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
use rustyline::completion::{Completer, FilenameCompleter, Pair};
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::{ValidationContext, ValidationResult, Validator};
use rustyline::{Context, Helper};
use std::borrow::Cow;
use std::path::PathBuf;

pub(crate) struct CommandHelper {
    commands: Vec<String>,
    file_commands: Vec<String>,
    file_completer: FileCompleter,
}

impl CommandHelper {
    pub(crate) fn new(commands: Vec<&str>, file_commands: Vec<&str>, cforge_dir: &str) -> Self {
        CommandHelper {
            commands: commands.iter().map(|s| s.to_string()).collect(),
            file_commands: file_commands.iter().map(|s| s.to_string()).collect(),
            file_completer: FileCompleter::new(cforge_dir),
        }
    }
}

struct FileCompleter {
    base_dir: PathBuf,
}

impl FileCompleter {
    fn new(base_dir: impl Into<PathBuf>) -> Self {
        FileCompleter {
            base_dir: base_dir.into(),
        }
    }
}

impl Completer for FileCompleter {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        original_pos: usize,
        ctx: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        let filename_completer = FilenameCompleter::new();

        if line.starts_with("/") {
            // For absolute paths, just use the FilenameCompleter directly
            filename_completer.complete(line, original_pos, ctx)
        } else {
            let absolute_path = self.base_dir.join(line);
            let absolute_pos = absolute_path.to_string_lossy().len();
            let (_, candidates) =
                filename_completer.complete(&absolute_path.to_string_lossy(), absolute_pos, ctx)?;

            let base_dir_str = self.base_dir.to_string_lossy().to_string();

            let modified_candidates: Vec<Pair> = candidates
                .into_iter()
                .map(|pair| {
                    let display = pair.display;
                    let new_display = if let Some(stripped) = display.strip_prefix(&base_dir_str) {
                        if let Some(stripped) = stripped.strip_prefix('/') {
                            stripped.to_string()
                        } else {
                            stripped.to_string()
                        }
                    } else {
                        display
                    };

                    let new_replacement =
                        if let Some(stripped) = pair.replacement.strip_prefix(&base_dir_str) {
                            if let Some(stripped) = stripped.strip_prefix('/') {
                                stripped.to_string()
                            } else {
                                stripped.to_string()
                            }
                        } else {
                            pair.replacement
                        };

                    Pair {
                        display: new_display,
                        replacement: new_replacement,
                    }
                })
                .collect();

            Ok((original_pos, modified_candidates))
        }
    }
}

impl Completer for CommandHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        ctx: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        if line.starts_with(":") {
            if line.contains(" ") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                let command = parts.get(0).unwrap_or(&"");

                // if the command is not a file command, return an empty list
                if !self.file_commands.contains(&command.to_string()) {
                    return Ok((pos, vec![]));
                }

                let arg = parts.get(1).unwrap_or(&"");
                let arg_start_pos = if arg.is_empty() {
                    line.len()
                } else {
                    line.find(arg).unwrap_or(line.len())
                };

                let res = self.file_completer.complete(arg, 0, ctx)?;

                Ok((arg_start_pos + res.0, res.1))
            } else {
                // Handle command completion
                let word_start = 1;
                let word = &line[word_start..pos];

                let matches: Vec<Pair> = self
                    .commands
                    .iter()
                    .filter(|cmd| cmd.starts_with(word) && cmd.len() > word.len())
                    .map(|cmd| Pair {
                        display: cmd.clone(),
                        replacement: cmd.clone(),
                    })
                    .collect();

                Ok((word_start, matches))
            }
        } else {
            Ok((pos, vec![]))
        }
    }
}

impl Hinter for CommandHelper {
    type Hint = String;

    fn hint(&self, line: &str, pos: usize, _ctx: &Context<'_>) -> Option<Self::Hint> {
        // Simple hinting - can be expanded as needed
        if line.starts_with(":") && !line.contains(" ") && pos == line.len() {
            let command = &line[1..];
            // Only show hints at the end of the line
            for cmd in &self.commands {
                if cmd.starts_with(command) && cmd != command && cmd.len() > command.len() {
                    return Some((&cmd[command.len()..]).to_string());
                }
            }
        }
        None
    }
}

impl Highlighter for CommandHelper {
    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
        Cow::Borrowed(line)
    }

    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Cow::Borrowed(hint)
    }
}

impl Validator for CommandHelper {
    fn validate(&self, _ctx: &mut ValidationContext) -> rustyline::Result<ValidationResult> {
        Ok(ValidationResult::Valid(None))
    }
}

// This is the key trait that combines all the above functionality
impl Helper for CommandHelper {}

#[cfg(test)]
mod tests {
    use super::*;
    use rustyline::history::DefaultHistory;
    use rustyline::Context;
    use std::collections::HashSet;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_command_helper_new() {
        let commands = vec!["help", "quit", "save"];
        let helper = CommandHelper::new(commands, vec![], "");

        assert_eq!(helper.commands, vec!["help", "quit", "save"]);
        assert!(helper.file_commands.is_empty());
    }

    #[test]
    fn test_command_with_arguments() {
        let commands = vec!["help", "quit", "save"];
        let helper = CommandHelper::new(commands, vec![], "");
        let history = DefaultHistory::new();
        let ctx = Context::new(&history);

        let (pos, matches) = helper.complete(":save file.txt", 12, &ctx).unwrap();
        assert_eq!(pos, 12);
        assert!(matches.is_empty());
    }

    #[test]
    fn test_no_matches() {
        let commands = vec!["help", "quit", "save"];
        let helper = CommandHelper::new(commands, vec![], "");
        let history = DefaultHistory::new();
        let ctx = Context::new(&history);

        let (pos, matches) = helper.complete(":xyz", 4, &ctx).unwrap();
        assert_eq!(pos, 1);
        assert!(matches.is_empty());
    }

    #[test]
    fn test_complete_empty_line() {
        let commands = vec!["help", "quit", "save"];
        let helper = CommandHelper::new(commands, vec![], "");
        let history = DefaultHistory::new();
        let ctx = Context::new(&history);

        let (pos, matches) = helper.complete("", 0, &ctx).unwrap();
        assert_eq!(pos, 0);
        assert!(matches.is_empty());
    }

    #[test]
    fn test_complete_non_command_line() {
        let commands = vec!["help", "quit", "save", "hey"];
        let helper = CommandHelper::new(commands, vec![], "");
        let history = DefaultHistory::new();
        let ctx = Context::new(&history);

        let (pos, matches) = helper.complete("Hey there", 0, &ctx).unwrap();
        assert_eq!(pos, 0);
        assert!(matches.is_empty());
    }

    #[test]
    fn test_hint_no_match() {
        let commands = vec!["help", "quit", "save"];
        let helper = CommandHelper::new(commands, vec![], "");
        let history = DefaultHistory::new();
        let ctx = Context::new(&history);

        let hint = helper.hint(":xyz", 4, &ctx);
        assert_eq!(hint, None);
    }

    #[test]
    fn test_hint_with_space() {
        let commands = vec!["help", "quit", "save"];
        let helper = CommandHelper::new(commands, vec![], "");
        let history = DefaultHistory::new();
        let ctx = Context::new(&history);

        let hint = helper.hint(":help ", 6, &ctx);
        assert_eq!(hint, None);
    }

    #[test]
    fn test_empty_commands_list() {
        let helper = CommandHelper::new(vec![], vec![], "");
        let history = DefaultHistory::new();
        let ctx = Context::new(&history);

        // Try to complete with an empty command list
        let (pos, matches) = helper.complete(":h", 2, &ctx).unwrap();
        assert_eq!(pos, 1);
        assert!(matches.is_empty());

        // Try to get a hint with an empty command list
        let hint = helper.hint(":h", 2, &ctx);
        assert_eq!(hint, None);
    }

    #[test]
    fn test_complete_command() {
        let commands = vec!["help", "quit", "save"];
        let helper = CommandHelper::new(commands, vec![], "");
        let history = DefaultHistory::new();
        let ctx = Context::new(&history);

        // Partial command
        let (pos, matches) = helper.complete(":h", 2, &ctx).unwrap();
        assert_eq!(pos, 1);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].display, "help");

        // Complete command
        let (pos, matches) = helper.complete(":help", 5, &ctx).unwrap();
        assert_eq!(pos, 1);
        assert!(matches.is_empty());
    }

    #[test]
    fn multiple_matches() {
        let commands = vec!["help", "quit", "switch", "sysprompt"];
        let helper = CommandHelper::new(commands, vec![], "");
        let history = DefaultHistory::new();
        let ctx = Context::new(&history);

        let (pos, matches) = helper.complete(":s", 2, &ctx).unwrap();
        assert_eq!(pos, 1);
        assert_eq!(matches.len(), 2);

        let match_strings: Vec<String> = matches.iter().map(|m| m.display.clone()).collect();
        assert!(match_strings.contains(&"switch".to_string()));
        assert!(match_strings.contains(&"sysprompt".to_string()));
    }

    #[test]
    fn test_hint() {
        let commands = vec!["help", "quit", "save"];
        let helper = CommandHelper::new(commands, vec![], "");
        let history = DefaultHistory::new();
        let ctx = Context::new(&history);

        // Partial command
        let hint = helper.hint(":h", 2, &ctx);
        assert_eq!(hint, Some("elp".to_string()));

        // Complete command
        let hint = helper.hint(":help", 5, &ctx);
        assert_eq!(hint, None);
    }

    #[test]
    fn test_highlighter() {
        let helper = CommandHelper::new(vec!["help"], vec![], "");

        // Test line highlighting (currently returns unchanged)
        let highlighted = helper.highlight("test line", 4);
        assert_eq!(highlighted, "test line");

        // Test hint highlighting (currently returns unchanged)
        let highlighted_hint = helper.highlight_hint("hint text");
        assert_eq!(highlighted_hint, "hint text");
    }

    #[test]
    fn test_file_completer() -> Result<(), Box<dyn std::error::Error>> {
        // Create a temporary directory structure for testing
        let temp_dir = TempDir::new()?;
        let base_path = temp_dir.path().to_path_buf();

        // Create some files and directories for testing
        fs::create_dir(base_path.join("dir1"))?;
        fs::create_dir(base_path.join("dir2"))?;
        fs::write(base_path.join("file1.txt"), b"content")?;
        fs::write(base_path.join("file2.rs"), b"content")?;
        fs::write(base_path.join("dir1").join("nested.txt"), b"content")?;

        // Create the file completer with the temp directory as base
        let completer = FileCompleter::new(base_path.clone());

        // Create a dummy context (not used in most implementations)
        let history = DefaultHistory::new();
        let ctx = Context::new(&history);

        // Test empty string completion (should list all files/dirs)
        let (pos, completions) = completer.complete("", 0, &ctx)?;

        // Check position
        assert_eq!(pos, 0, "Position should be 0 for empty string");

        // Convert completions to a set of strings for easier comparison
        let completion_set: HashSet<String> = completions
            .iter()
            .map(|pair| pair.display.clone())
            .collect();

        // Verify all expected files/directories are in the completions
        assert!(completion_set.contains("dir1"), "Should contain dir1");
        assert!(completion_set.contains("dir2"), "Should contain dir2");
        assert!(
            completion_set.contains("file1.txt"),
            "Should contain file1.txt"
        );
        assert!(
            completion_set.contains("file2.rs"),
            "Should contain file2.rs"
        );

        // Test partial completion
        let (pos, completions) = completer.complete("file", 4, &ctx)?;

        assert_eq!(pos, 4, "Position should be 0 for partial completion");
        assert_eq!(
            completions.len(),
            2,
            "Should find 2 files starting with 'file'"
        );

        let completion_set: HashSet<String> = completions
            .iter()
            .map(|pair| pair.display.clone())
            .collect();

        assert!(
            completion_set.contains("file1.txt"),
            "Should contain file1.txt"
        );
        assert!(
            completion_set.contains("file2.rs"),
            "Should contain file2.rs"
        );

        // Test directory completion
        let (pos, completions) = completer.complete("dir1/", 5, &ctx)?;

        assert_eq!(pos, 5, "Position should be 5 for directory completion");
        assert_eq!(completions.len(), 1, "Should find 1 file in dir1");
        assert_eq!(
            completions[0].display, "nested.txt",
            "Should find nested.txt in dir1"
        );

        // Test completion with non-existent path
        let (pos, completions) = completer.complete("nonexistent", 11, &ctx)?;

        assert_eq!(pos, 11, "Position should be 0 for non-existent path");
        assert_eq!(
            completions.len(),
            0,
            "Should find 0 files for non-existent path"
        );

        Ok(())
    }

    #[test]
    fn test_file_completer_shared_path() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let base_path = temp_dir.path().to_path_buf();
        let history = DefaultHistory::new();
        let ctx = Context::new(&history);

        fs::create_dir(base_path.join("tests"))?;
        fs::write(base_path.join("testing.md"), b"content")?;

        let completer = FileCompleter::new(base_path.clone());
        let (pos, completions) = completer.complete("te", 2, &ctx)?;

        assert_eq!(pos, 2, "Position should be 0 for empty string");
        assert_eq!(
            completions.len(),
            2,
            "Should find 2 files starting with 'te'"
        );
        assert_eq!(
            completions[0].display, "testing.md",
            "Should find testing.md"
        );

        Ok(())
    }

    #[test]
    fn test_file_completer_custom() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let base_path = temp_dir.path().to_path_buf();
        let history = DefaultHistory::new();
        let ctx = Context::new(&history);

        fs::create_dir(base_path.join("tests"))?;
        fs::write(base_path.join("testing.md"), b"content")?;

        let completer = FileCompleter::new(base_path.clone());
        let (pos, completions) = completer.complete("te", 2, &ctx)?;

        assert_eq!(pos, 2, "Position should be 0 for empty string");
        assert_eq!(
            completions.len(),
            2,
            "Should find 2 files starting with 'te'"
        );
        assert_eq!(
            completions[0].display, "testing.md",
            "Should find testing.md"
        );

        Ok(())
    }
}
