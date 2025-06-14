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
use rustyline::completion::{Completer, Pair};
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::{ValidationContext, ValidationResult, Validator};
use rustyline::{Context, Helper};
use std::borrow::Cow;

pub(crate) struct CommandHelper {
    commands: Vec<String>,
    file_commands: Vec<String>,
}

impl CommandHelper {
    pub(crate) fn new(commands: Vec<&str>) -> Self {
        CommandHelper {
            commands: commands.iter().map(|s| s.to_string()).collect(),
            file_commands: vec![],
        }
    }
}

impl Completer for CommandHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        if line.starts_with(":") {
            let parts: Vec<&str> = line.split(' ').collect();
            let command = parts[0];

            if line.contains(" ") {
                // Handle args
                Ok((pos, vec![]))
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

    #[test]
    fn test_command_helper_new() {
        let commands = vec!["help", "quit", "save"];
        let helper = CommandHelper::new(commands);

        assert_eq!(helper.commands, vec!["help", "quit", "save"]);
        assert!(helper.file_commands.is_empty());
    }

    #[test]
    fn test_command_with_arguments() {
        let commands = vec!["help", "quit", "save"];
        let helper = CommandHelper::new(commands);
        let history = DefaultHistory::new();
        let ctx = Context::new(&history);

        let (pos, matches) = helper.complete(":save file.txt", 12, &ctx).unwrap();
        assert_eq!(pos, 12);
        assert!(matches.is_empty());
    }

    #[test]
    fn test_no_matches() {
        let commands = vec!["help", "quit", "save"];
        let helper = CommandHelper::new(commands);
        let history = DefaultHistory::new();
        let ctx = Context::new(&history);

        let (pos, matches) = helper.complete(":xyz", 4, &ctx).unwrap();
        assert_eq!(pos, 1);
        assert!(matches.is_empty());
    }

    #[test]
    fn test_complete_empty_line() {
        let commands = vec!["help", "quit", "save"];
        let helper = CommandHelper::new(commands);
        let history = DefaultHistory::new();
        let ctx = Context::new(&history);

        let (pos, matches) = helper.complete("", 0, &ctx).unwrap();
        assert_eq!(pos, 0);
        assert!(matches.is_empty());
    }

    #[test]
    fn test_complete_non_command_line() {
        let commands = vec!["help", "quit", "save", "hey"];
        let helper = CommandHelper::new(commands);
        let history = DefaultHistory::new();
        let ctx = Context::new(&history);

        let (pos, matches) = helper.complete("Hey there", 0, &ctx).unwrap();
        assert_eq!(pos, 0);
        assert!(matches.is_empty());
    }

    #[test]
    fn test_hint_no_match() {
        let commands = vec!["help", "quit", "save"];
        let helper = CommandHelper::new(commands);
        let history = DefaultHistory::new();
        let ctx = Context::new(&history);

        let hint = helper.hint(":xyz", 4, &ctx);
        assert_eq!(hint, None);
    }

    #[test]
    fn test_hint_with_space() {
        let commands = vec!["help", "quit", "save"];
        let helper = CommandHelper::new(commands);
        let history = DefaultHistory::new();
        let ctx = Context::new(&history);

        let hint = helper.hint(":help ", 6, &ctx);
        assert_eq!(hint, None);
    }

    #[test]
    fn test_empty_commands_list() {
        let helper = CommandHelper::new(vec![]);
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
        let helper = CommandHelper::new(commands);
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
        let helper = CommandHelper::new(commands);
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
        let helper = CommandHelper::new(commands);
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
        let helper = CommandHelper::new(vec!["help"]);

        // Test line highlighting (currently returns unchanged)
        let highlighted = helper.highlight("test line", 4);
        assert_eq!(highlighted, "test line");

        // Test hint highlighting (currently returns unchanged)
        let highlighted_hint = helper.highlight_hint("hint text");
        assert_eq!(highlighted_hint, "hint text");
    }
}
