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

use crate::command::commands::{CommandParams, CommandResult, CommandStruct, FileCommandDirectory};
use std::collections::HashMap;
use std::io;

pub(crate) fn new<'a>(default_prefixes: &HashMap<String, String>) -> (String, CommandStruct<'a>) {
    (
        "switch".to_string(),
        CommandStruct::new(
            "switch",
            "Switch to a different history file. Either relative to the data directory or absolute path. Creates the file if it doesn't exist.",
            Some(":switch <history file>"),
            Some(FileCommandDirectory::Cforge),
            switch_command,
            default_prefixes.get("switch").cloned(),
        ),
    )
}

pub(crate) fn command<'a>(default_prefixes: &HashMap<String, String>) -> (String, CommandStruct<'a>) {
    new(default_prefixes)
}

pub(crate) fn switch_command(command_params: CommandParams) -> io::Result<CommandResult> {
    match command_params.args.first() {
        Some(new_history_file) => Ok(CommandResult::SwitchHistory(new_history_file.to_string())),
        _ => {
            println!("Error: No history file specified. Usage: :switch <history_file>");
            Ok(CommandResult::Continue)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::ChatClient;
    use crate::history_file::HistoryFile;
    use crate::test_support::make_mock_client;
    use std::{fs, io};
    use tempfile::TempDir;

    fn setup_test_environment() -> (Box<dyn ChatClient>, HistoryFile, TempDir, String) {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path().to_str().unwrap().to_string();
        let chat_client: Box<dyn ChatClient> = make_mock_client();
        let history_path = format!("{}/test-history.txt", dir_path);
        fs::write(&history_path, "Test conversation content").unwrap();
        let history = HistoryFile::new("test-history.txt".to_string(), dir_path.clone()).unwrap();
        (chat_client, history, temp_dir, dir_path)
    }

    #[test]
    fn test_switch_command() -> io::Result<()> {
        let (mut client, mut history, _tmp, dir_path) = setup_test_environment();
        let new_history_file = "new-history.txt";
        fs::write(format!("{}/{}", dir_path, new_history_file), "New history content")?;
        let args = vec![new_history_file.to_string()];
        let params = CommandParams::new(args, &mut client, &mut history, dir_path);
        let result = switch_command(params)?;
        if let CommandResult::SwitchHistory(filename) = result {
            assert_eq!(filename, new_history_file);
        } else {
            panic!("Expected SwitchHistory result but got something else");
        }
        Ok(())
    }

    #[test]
    fn test_switch_command_with_no_args() -> io::Result<()> {
        let (mut client, mut history, _tmp, dir_path) = setup_test_environment();
        let params = CommandParams::new(vec![], &mut client, &mut history, dir_path);
        let result = switch_command(params)?;
        assert!(matches!(result, CommandResult::Continue));
        Ok(())
    }
}
