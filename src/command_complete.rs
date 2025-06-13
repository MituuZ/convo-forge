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
