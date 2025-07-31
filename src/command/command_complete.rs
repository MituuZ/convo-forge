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
use crate::command::commands::FileCommandDirectory;
use rustyline::completion::{Completer, FilenameCompleter, Pair};
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::{ValidationContext, ValidationResult, Validator};
use rustyline::{Context, Helper};
use std::borrow::Cow;
use std::path::PathBuf;

pub struct CommandHelper {
    commands: Vec<(String, Option<String>)>,
    file_commands: Vec<(String, FileCommandDirectory)>,
    file_completer: FileCompleter,
}

impl CommandHelper {
    pub(crate) fn new(
        commands: Vec<(String, Option<String>)>,
        file_commands: Vec<(String, FileCommandDirectory)>,
        cforge_dir: &str,
        knowledge_dir: &str,
        prompt_dir: &str,
    ) -> Self {
        CommandHelper {
            commands,
            file_commands,
            file_completer: FileCompleter::new(cforge_dir, knowledge_dir, prompt_dir),
        }
    }
}

struct FileCompleter {
    base_dir: PathBuf,
    knowledge_dir: PathBuf,
    prompt_dir: PathBuf,
    filename_completer: FilenameCompleter,
}

impl FileCompleter {
    fn new(
        base_dir: impl Into<PathBuf>,
        knowledge_dir: impl Into<PathBuf>,
        prompt_dir: impl Into<PathBuf>,
    ) -> Self {
        FileCompleter {
            base_dir: base_dir.into(),
            knowledge_dir: knowledge_dir.into(),
            prompt_dir: prompt_dir.into(),
            filename_completer: FilenameCompleter::new(),
        }
    }
}

impl Completer for FileCompleter {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        _: usize,
        ctx: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        if let Some(actual_query) = line.strip_prefix("@") {
            if let Some((prefix, subpath)) = actual_query.split_once("/") {
                let full_path = match prefix {
                    "c" | "k" | "p" => {
                        let base_dir = match prefix {
                            "c" => &self.base_dir,
                            "k" => &self.knowledge_dir,
                            "p" => &self.prompt_dir,
                            _ => unreachable!()
                        };
                        if subpath.is_empty() {
                            base_dir.clone()
                        } else {
                            base_dir.join(subpath)
                        }
                    }
                    _ => return Ok((0, vec![])),
                };
                let full_path_str = full_path.to_string_lossy();
                // The cursor is at the end of the full path string now
                let pos = full_path_str.len();

                return self.filename_completer.complete(&full_path_str, pos, ctx);
            }
        }

        self.filename_completer.complete(line, line.len(), ctx)
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

                // if the command is not a file command, return an empty list
                match parts.first().unwrap_or(&"").strip_prefix(":") {
                    Some(command) => {
                        if !self.file_commands.iter().any(|entry| {
                            let (cmd, _) = entry;
                            cmd == command
                        }) {
                            return Ok((pos, vec![]));
                        }
                    }
                    None => {
                        return Ok((pos, vec![]));
                    }
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
                    .filter(|tuple| tuple.0.starts_with(word) && tuple.0.len() > word.len())
                    .map(|(cmd, default_alias)| Pair {
                        display: format!("{} {}", cmd, default_alias.as_deref().unwrap_or("")),
                        replacement: format!("{} {}", cmd, default_alias.as_deref().unwrap_or("")),
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
            for (cmd, _) in &self.commands {
                if cmd.starts_with(command) && cmd != command && cmd.len() > command.len() {
                    return Some(cmd[command.len()..].to_string());
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
    use rustyline::completion::Candidate;
    use rustyline::hint::Hint;
    use rustyline::history::DefaultHistory;
    use rustyline::Context;
    use std::collections::HashSet;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_command_helper_new() {
        let helper = create_command_helper();

        assert_eq!(
            helper.commands,
            vec![
                ("help".to_string(), None),
                ("quit".to_string(), None),
                ("save".to_string(), None)
            ]
        );
        assert!(helper.file_commands.is_empty());
    }

    #[test]
    fn test_command_with_arguments() {
        let helper = create_command_helper();
        let history = DefaultHistory::new();
        let ctx = Context::new(&history);

        let (pos, matches) = helper.complete(":save file.txt", 12, &ctx).unwrap();
        assert_eq!(pos, 12);
        assert!(matches.is_empty());
    }

    #[test]
    fn test_no_matches() {
        let helper = create_command_helper();
        let history = DefaultHistory::new();
        let ctx = Context::new(&history);

        let (pos, matches) = helper.complete(":xyz", 4, &ctx).unwrap();
        assert_eq!(pos, 1);
        assert!(matches.is_empty());
    }

    #[test]
    fn test_complete_empty_line() {
        let helper = create_command_helper();

        let history = DefaultHistory::new();
        let ctx = Context::new(&history);

        let (pos, matches) = helper.complete("", 0, &ctx).unwrap();
        assert_eq!(pos, 0);
        assert!(matches.is_empty());
    }

    #[test]
    fn test_complete_non_command_line() {
        let commands = vec![
            ("help".to_string(), None),
            ("quit".to_string(), None),
            ("save".to_string(), None),
            ("hey".to_string(), None),
        ];
        let helper = CommandHelper::new(commands, vec![], "", "", "");
        let history = DefaultHistory::new();
        let ctx = Context::new(&history);

        let (pos, matches) = helper.complete("Hey there", 0, &ctx).unwrap();
        assert_eq!(pos, 0);
        assert!(matches.is_empty());
    }

    #[test]
    fn test_hint_no_match() {
        let helper = create_command_helper();
        let history = DefaultHistory::new();
        let ctx = Context::new(&history);

        let hint = helper.hint(":xyz", 4, &ctx);
        assert_eq!(hint, None);
    }

    #[test]
    fn test_hint_with_space() {
        let helper = create_command_helper();
        let history = DefaultHistory::new();
        let ctx = Context::new(&history);

        let hint = helper.hint(":help ", 6, &ctx);
        assert_eq!(hint, None);
    }

    #[test]
    fn test_empty_commands_list() {
        let helper = CommandHelper::new(vec![], vec![], "", "", "");
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
        let helper = create_command_helper();
        let history = DefaultHistory::new();
        let ctx = Context::new(&history);

        // Partial command
        let (pos, matches) = helper.complete(":h", 2, &ctx).unwrap();
        assert_eq!(pos, 1);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].display, "help ");

        // Complete command
        let (pos, matches) = helper.complete(":help", 5, &ctx).unwrap();
        assert_eq!(pos, 1);
        assert!(matches.is_empty());
    }

    #[test]
    fn multiple_matches() {
        let commands = vec![
            ("help".to_string(), None),
            ("quit".to_string(), None),
            ("switch".to_string(), None),
            ("sysprompt".to_string(), None),
        ];
        let helper = CommandHelper::new(commands, vec![], "", "", "");
        let history = DefaultHistory::new();
        let ctx = Context::new(&history);

        let (pos, matches) = helper.complete(":s", 2, &ctx).unwrap();
        assert_eq!(pos, 1);
        assert_eq!(matches.len(), 2);

        let match_strings: Vec<String> = matches.iter().map(|m| m.display.clone()).collect();
        assert!(match_strings.contains(&"switch ".to_string()));
        assert!(match_strings.contains(&"sysprompt ".to_string()));
    }

    #[test]
    fn test_hint() {
        let helper = create_command_helper();
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
        let helper = CommandHelper::new(vec![("help".to_string(), None)], vec![], "", "", "");

        // Test line highlighting (currently returns unchanged)
        let highlighted = helper.highlight("test line", 4);
        assert_eq!(highlighted, "test line");

        // Test hint highlighting (currently returns unchanged)
        let highlighted_hint = helper.highlight_hint("hint text");
        assert_eq!(highlighted_hint, "hint text");
    }

    #[test]
    fn test_file_completer_base_dir() -> Result<(), Box<dyn std::error::Error>> {
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
        let completer = FileCompleter::new(base_path.clone(), "", "");

        // Create a dummy context (not used in most implementations)
        let history = DefaultHistory::new();
        let ctx = Context::new(&history);

        // Test empty string completion (should list all files/dirs)
        let (pos, completions) = completer.complete("@c/", 0, &ctx)?;
        let first_replacement = completions[0].replacement.clone();

        assert_eq!(pos, 0, "Position should be 0 for empty string");

        assert_eq!(first_replacement, format!("{}{}", base_path.display(), std::path::MAIN_SEPARATOR), "Replacement should be base path");

        // Next completion should return the actual contents
        let (_pos, completions) = completer.complete(&first_replacement, 0, &ctx)?;

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
        let (pos, completions) = completer.complete("@c/file", 4, &ctx)?;

        assert_eq!(pos, 0, "Position should be 0 for partial completion");
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
        let (pos, completions) = completer.complete("@c/dir1/", 5, &ctx)?;

        assert_eq!(pos, 0, "Position should be 0 for directory completion");
        assert_eq!(completions.len(), 1, "Should find 1 file in dir1");
        assert_eq!(
            completions[0].display, "nested.txt",
            "Should find nested.txt in dir1"
        );

        // Test completion with non-existent path
        let (pos, completions) = completer.complete("nonexistent", 11, &ctx)?;

        assert_eq!(pos, 0, "Position should be 0 for non-existent path");
        assert_eq!(
            completions.len(),
            0,
            "Should find 0 files for non-existent path"
        );

        Ok(())
    }

    #[test]
    fn test_file_completer_shared_path_base_dir() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let base_path = temp_dir.path().to_path_buf();
        let history = DefaultHistory::new();
        let ctx = Context::new(&history);

        fs::create_dir(base_path.join("tests"))?;
        fs::write(base_path.join("testing.md"), b"content")?;

        let completer = FileCompleter::new(base_path.clone(), "", "");
        let (pos, completions) = completer.complete("@c/te", 2, &ctx)?;

        assert_eq!(pos, 0, "Position should be 0 for empty string");
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

        let completer = FileCompleter::new(base_path.clone(), "", "");
        let (pos, completions) = completer.complete("@c/te", 2, &ctx)?;

        assert_eq!(pos, 0, "Position should be 0 for empty string");
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
    fn test_command_prefix_handling() {
        let commands = vec![
            ("help".to_string(), None),
            ("hello".to_string(), Some("@c/".into())),
        ];
        let helper = CommandHelper::new(commands, vec![], "", "", "");
        let history = DefaultHistory::new();
        let ctx = Context::new(&history);

        let (pos, matches) = helper.complete(":h", 2, &ctx).unwrap();
        assert_eq!(pos, 1);
        assert_eq!(matches.len(), 2);

        let completions: Vec<String> = matches.iter().map(|m| m.display.clone()).collect();
        assert!(completions.contains(&"help ".to_string()));
        assert!(completions.contains(&"hello @c/".to_string()));
    }

    fn create_command_helper() -> CommandHelper {
        let commands = vec![
            ("help".to_string(), None),
            ("quit".to_string(), None),
            ("save".to_string(), None),
        ];
        CommandHelper::new(commands, vec![], "", "", "")
    }
}
