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

use crate::command::commands::{CommandParams, CommandResult, CommandStruct};
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io;

pub(crate) fn new<'a>(_default_prefixes: &HashMap<String, String>) -> (String, CommandStruct<'a>) {
    (
        "clear".to_string(),
        CommandStruct::new(
            "clear",
            "Clear the current history file (empties its contents).",
            Some(":clear"),
            None,
            clear_command,
            None,
        ),
    )
}

pub(crate) fn command<'a>(default_prefixes: &HashMap<String, String>) -> (String, CommandStruct<'a>) {
    new(default_prefixes)
}

/// Truncates the history file and reloads the in-memory content.
pub(crate) fn clear_command(command_params: CommandParams) -> io::Result<CommandResult> {
    let path = command_params.history.path.clone();
    let _ = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(&path)?;

    command_params.history.reload_content();

    println!("History cleared: {}", command_params.history.filename);

    Ok(CommandResult::Continue)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::setup_test_environment;
    use std::fs;
    use std::io;

    #[test]
    fn test_clear_command_empties_history() -> io::Result<()> {
        let (mut client, mut history, _tmp, dir_path) = setup_test_environment();

        history.append_user_input("Hello world")?;
        assert!(!history.get_content().is_empty());

        let params = CommandParams::new(vec![], &mut client, &mut history, dir_path.clone());
        let result = clear_command(params)?;
        assert!(matches!(result, CommandResult::Continue));

        assert_eq!(history.get_content(), "");

        let disk_content = fs::read_to_string(history.path.clone())?;
        assert_eq!(disk_content, "");

        Ok(())
    }
}
