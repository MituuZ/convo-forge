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

pub(crate) enum UserInput {
    Command(Command),
    Prompt(String),
}

pub(crate) struct Command {
    pub(crate) name: String,
    pub(crate) args: Vec<String>,
}

impl UserInput {
    pub(crate) fn parse(input: &str) -> Self {
        let input = input.trim();
        if input.starts_with(":") {
            let parts: Vec<&str> = input.split_whitespace().collect();
            let name = parts[0].trim_start_matches(":").to_lowercase();

            let args = parts[1..].iter().map(|s| s.to_string()).collect();

            UserInput::Command(Command { name, args })
        } else {
            UserInput::Prompt(input.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_command() {
        let input = ":help arg1 arg2";
        match UserInput::parse(input) {
            UserInput::Command(cmd) => {
                assert_eq!(cmd.name, "help");
                assert_eq!(cmd.args, vec!["arg1", "arg2"]);
            }
            _ => panic!("Expected Command, got Prompt"),
        }
    }

    #[test]
    fn test_parse_prompt() {
        let input = "This is a regular prompt";
        match UserInput::parse(input) {
            UserInput::Prompt(text) => {
                assert_eq!(text, "This is a regular prompt");
            }
            _ => panic!("Expected Prompt, got Command"),
        }
    }

    #[test]
    fn test_parse_empty_input() {
        let input = "   ";
        match UserInput::parse(input) {
            UserInput::Prompt(text) => {
                assert_eq!(text, "");
            }
            _ => panic!("Expected Prompt, got Command"),
        }
    }

    #[test]
    fn test_parse_empty_command() {
        let input = ":";
        match UserInput::parse(input) {
            UserInput::Command(cmd) => {
                assert_eq!(cmd.name, "");
            }
            _ => panic!("Expected Prompt, got Command"),
        }
    }
}
