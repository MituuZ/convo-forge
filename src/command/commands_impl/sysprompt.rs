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
use std::io;

pub(crate) fn new<'a>(_default_prefixes: &HashMap<String, String>) -> (String, CommandStruct<'a>) {
    (
        "sysprompt".to_string(),
        CommandStruct::new(
            "sysprompt",
            "Set the system prompt for current session",
            Some(":sysprompt <prompt>"),
            None,
            sysprompt_command,
            None,
        ),
    )
}

pub(crate) fn command<'a>(default_prefixes: &HashMap<String, String>) -> (String, CommandStruct<'a>) {
    new(default_prefixes)
}

pub(crate) fn sysprompt_command(command_params: CommandParams) -> io::Result<CommandResult> {
    command_params
        .chat_client
        .update_system_prompt(command_params.args.join(" "));
    Ok(CommandResult::Continue)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::{ChatClient, ChatResponse};
    use crate::history_file::HistoryFile;
    use serde_json::Value;
    use std::io;
    use tempfile::TempDir;

    struct MockClient {
        system_prompt: String,
    }
    impl ChatClient for MockClient {
        fn generate_response(&self, _: Value, _: &str, _: Option<&str>) -> io::Result<ChatResponse> {
            Ok(ChatResponse { content: String::new(), tool_calls: None })
        }
        fn generate_tool_response(&self, _: Value) -> io::Result<ChatResponse> { unreachable!() }
        fn model_context_size(&self) -> Option<usize> { None }
        fn model_supports_tools(&self) -> bool { false }
        fn update_system_prompt(&mut self, sp: String) { self.system_prompt = sp; }
        fn system_prompt(&self) -> String { self.system_prompt.clone() }
    }

    fn setup_test_environment() -> (Box<dyn ChatClient>, HistoryFile, TempDir, String) {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path().to_str().unwrap().to_string();
        let chat_client: Box<dyn ChatClient> = Box::new(MockClient { system_prompt: String::new() });
        let history = HistoryFile::new("test-history.txt".to_string(), dir_path.clone()).unwrap();
        (chat_client, history, temp_dir, dir_path)
    }

    #[test]
    fn test_sysprompt_command() -> io::Result<()> {
        let (mut chat_client, mut history, _temp_dir, dir_path) = setup_test_environment();
        let new_system_prompt = "This is a test system prompt";
        let initial_system_prompt = chat_client.system_prompt().clone();
        let args: Vec<String> = new_system_prompt.split_whitespace().map(|s| s.to_string()).collect();
        let params = CommandParams::new(args, &mut chat_client, &mut history, dir_path);
        assert_ne!(initial_system_prompt, new_system_prompt);
        let result = sysprompt_command(params)?;
        assert!(matches!(result, CommandResult::Continue));
        assert_eq!(chat_client.system_prompt(), new_system_prompt);
        Ok(())
    }
}
