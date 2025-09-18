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

use crate::command::commands::CommandResult::HandlePrompt;
use crate::command::commands::{CommandParams, CommandResult, CommandStruct, FileCommandDirectory};
use std::collections::HashMap;
use std::io;
use std::path::PathBuf;

pub(crate) fn new<'a>(default_prefixes: &HashMap<String, String>) -> (String, CommandStruct<'a>) {
    (
        "prompt".to_string(),
        CommandStruct::new(
            "prompt",
            r"Select or edit a prompt file. Either relative to the prompt directory or asolute path. Creates the file if it doesn't exist.",
            Some(
                r":prompt <prompt file>
            <actual prompt to use with the file>",
            ),
            Some(FileCommandDirectory::Prompt),
            prompt_command,
            default_prefixes.get("prompt").cloned(),
        ),
    )
}

pub(crate) fn command<'a>(default_prefixes: &HashMap<String, String>) -> (String, CommandStruct<'a>) {
    new(default_prefixes)
}

pub(crate) fn prompt_command(command_params: CommandParams) -> io::Result<CommandResult> {
    match command_params.args.first() {
        None => {
            eprintln!("Error: No prompt file specified. Usage: :prompt <prompt_file>");
            Ok(CommandResult::Continue)
        }
        Some(prompt_file) => {
            let user_prompt = if command_params.args.len() > 1 {
                Some(command_params.args[1..].join(" "))
            } else {
                None
            };

            Ok(HandlePrompt(PathBuf::from(prompt_file), user_prompt))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::commands::CommandResult::HandlePrompt;
    use crate::test_support::setup_test_environment;
    use std::io;

    #[test]
    fn test_prompt_command_no_input() -> io::Result<()> {
        let (mut client, mut history, _tmp, dir_path) = setup_test_environment();
        let args: Vec<String> = vec![];
        let params = CommandParams::new(args, &mut client, &mut history, dir_path);
        let result = prompt_command(params)?;
        assert!(matches!(result, CommandResult::Continue));
        Ok(())
    }

    #[test]
    fn test_prompt_command_edit_prompt_file() -> io::Result<()> {
        let (mut client, mut history, _tmp, dir_path) = setup_test_environment();
        let input = "prompt_file";
        let args: Vec<String> = input.split_whitespace().map(|s| s.to_string()).collect();
        let params = CommandParams::new(args, &mut client, &mut history, dir_path);
        let result = prompt_command(params)?;
        if let HandlePrompt(file, user_prompt) = result {
            assert_eq!(Some(user_prompt), Some(None));
            assert_eq!(file, PathBuf::from(input));
        } else {
            panic!("Expected HandlePrompt result but got something else");
        }
        Ok(())
    }

    #[test]
    fn test_prompt_command() -> io::Result<()> {
        let (mut client, mut history, _tmp, dir_path) = setup_test_environment();
        let test_prompt = "prompt_file This is a test prompt";
        let args: Vec<String> = test_prompt.split_whitespace().map(|s| s.to_string()).collect();
        let expected_prompt = Some(args[1..].join(" "));
        let expected_file = PathBuf::from("prompt_file");
        let params = CommandParams::new(args, &mut client, &mut history, dir_path);
        let result = prompt_command(params)?;
        if let HandlePrompt(file, user_prompt) = result {
            assert_eq!(Some(user_prompt), Some(expected_prompt));
            assert_eq!(file, expected_file);
        } else {
            panic!("Expected HandlePrompt result but got something else");
        }
        Ok(())
    }
}
