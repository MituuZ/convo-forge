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

use crate::api::ChatApi;
use crate::history_file::HistoryFile;
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
use std::{env, fs, io};

pub enum CommandResult {
    Continue,
    Quit,
    SwitchHistory(String),
    SwitchContext(Option<PathBuf>),
}

pub struct CommandParams<'a> {
    pub(crate) args: Vec<String>,
    chat_api: &'a mut Box<dyn ChatApi>,
    history: &'a mut HistoryFile,
    cforge_dir: String,
}

impl<'a> CommandParams<'a> {
    pub fn new(
        args: Vec<String>,
        chat_api: &'a mut Box<dyn ChatApi>,
        history: &'a mut HistoryFile,
        cforge_dir: String,
    ) -> Self {
        CommandParams {
            args,
            chat_api,
            history,
            cforge_dir,
        }
    }
}

type CommandFn = fn(CommandParams) -> io::Result<CommandResult>;

pub struct CommandStruct<'a> {
    pub(crate) command_string: &'a str,
    description: &'a str,
    command_example: Option<&'a str>,
    pub(crate) file_command: Option<FileCommand>,
    pub(crate) command_fn: CommandFn,
    pub(crate) default_prefix: Option<String>,
}

#[derive(Clone, Debug)]
pub enum FileCommand {
    KnowledgeDir,
    CforgeDir,
}

impl<'a> CommandStruct<'a> {
    pub fn new(
        command_string: &'a str,
        description: &'a str,
        command_example: Option<&'a str>,
        file_command: Option<FileCommand>,
        command_fn: CommandFn,
        default_prefix: Option<String>,
    ) -> Self {
        CommandStruct {
            command_string,
            command_example,
            description,
            file_command,
            command_fn,
            default_prefix: default_prefix,
        }
    }

    pub fn execute(&self, params: CommandParams) -> io::Result<CommandResult> {
        (self.command_fn)(params)
    }

    fn display(&self) -> String {
        match self.command_example {
            Some(example) => format!(
                "{:<12} - {}\n            {}",
                self.command_string, self.description, example
            ),
            None => format!("{:<12} - {}", self.command_string, self.description),
        }
    }
}

/// Helper function to create a new command struct as a tuple for the registry
fn cmd<'a>(
    name: &'a str,
    description: &'a str,
    command_example: Option<&'a str>,
    file_command: Option<FileCommand>,
    execute_fn: fn(CommandParams) -> io::Result<CommandResult>,
    default_prefix: Option<String>,
) -> (String, CommandStruct<'a>) {
    (
        name.to_string(),
        CommandStruct::new(name, description, command_example, file_command, execute_fn, default_prefix),
    )
}

pub(crate) fn create_command_registry<'a>(default_prefixes: HashMap<String, String>) -> HashMap<String, CommandStruct<'a>> {
    HashMap::from([
        cmd("q", "Exit the program", None, None, quit_command, None),
        cmd(
            "list",
            "List files in the cforge directory. \
                    Optionally, you can provide a pattern to filter the results.",
            Some(":list <optional pattern>"),
            Some(FileCommand::CforgeDir),
            list_command,
            default_prefixes.get("list").cloned(),
        ),
        cmd(
            "switch",
            "Switch to a different history file. \
                    Either relative to cforge_dir or absolute path. Creates the file if it doesn't exist.",
            Some(":switch <history file>"),
            Some(FileCommand::CforgeDir),
            switch_command,
            default_prefixes.get("switch").cloned(),
        ),
        cmd("help", "Show this help message", None, None, help_command, None),
        cmd(
            "edit",
            "Open the history file in your editor",
            None,
            None,
            edit_command,
            None,
        ),
        cmd(
            "sysprompt",
            "Set the system prompt for current session",
            Some(":sysprompt <prompt>"),
            None,
            sysprompt_command,
            None,
        ),
        cmd(
            "context",
            "Set or unset current context file",
            Some(":context <optional path>"),
            Some(FileCommand::KnowledgeDir),
            context_file_command,
            default_prefixes.get("context").cloned(),
        ),
        cmd(
            "switch",
            "Switch to a different history file. \
                    Either relative to cforge_dir or absolute path. Creates the file if it doesn't exist.",
            Some(":switch <history file>"),
            Some(FileCommand::CforgeDir),
            switch_command,
            default_prefixes.get("switch").cloned(),
        ),
    ])
}

fn quit_command(command_params: CommandParams) -> io::Result<CommandResult> {
    println!(
        "Ending conversation. All interactions saved to '{}'",
        command_params.history.filename
    );
    Ok(CommandResult::Quit)
}

fn list_command(command_params: CommandParams) -> io::Result<CommandResult> {
    let empty_string = String::from("");
    let pattern = command_params.args.first().unwrap_or(&empty_string);

    fn list_dir_contents(dir: &str, pattern: &str, cforge_dir: &str) -> io::Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if (pattern.is_empty() || path.display().to_string().contains(pattern))
                && !path.is_dir()
            {
                match path.display().to_string().strip_prefix(cforge_dir) {
                    None => println!("{}", path.display()),
                    Some(ds) => {
                        let mut cleaned_ds = ds.to_string();
                        if cleaned_ds.starts_with('/') {
                            cleaned_ds = cleaned_ds[1..].to_string();
                        }
                        println!("{cleaned_ds}")
                    }
                }
            }
            if path.is_dir() {
                list_dir_contents(path.to_str().unwrap(), pattern, cforge_dir)?;
            }
        }
        Ok(())
    }

    list_dir_contents(
        &command_params.cforge_dir,
        pattern,
        &command_params.cforge_dir,
    )?;
    Ok(CommandResult::Continue)
}

fn help_command(_command_params: CommandParams) -> io::Result<CommandResult> {
    let temp_map = HashMap::new();
    let registry = create_command_registry(temp_map);
    let mut commands: Vec<&CommandStruct> = registry.values().collect();

    commands.sort_by(|a, b| {
        a.file_command
            .is_some()
            .cmp(&b.file_command.is_some())
            .then(a.command_string.cmp(b.command_string))
    });

    // Print regular command first
    println!("General command:");
    for cmd in &commands {
        if cmd.file_command.is_none() {
            println!("{}", cmd.display());
        }
    }

    // Then print file command
    println!("\nFile command (supports file completion):");
    for cmd in &commands {
        if cmd.file_command.is_some() {
            println!("{}", cmd.display());
        }
    }

    Ok(CommandResult::Continue)
}

fn switch_command(command_params: CommandParams) -> io::Result<CommandResult> {
    match command_params.args.first() {
        Some(new_history_file) => Ok(CommandResult::SwitchHistory(new_history_file.to_string())),
        _ => {
            println!("Error: No history file specified. Usage: :switch <history_file>");
            Ok(CommandResult::Continue)
        }
    }
}

fn edit_command(command_params: CommandParams) -> io::Result<CommandResult> {
    let history = command_params.history;
    let editor = env::var("EDITOR")
        .or_else(|_| env::var("VISUAL"))
        .unwrap_or_else(|_| {
            if cfg!(target_os = "windows") {
                "notepad".to_string()
            } else {
                "vi".to_string()
            }
        });

    let status = Command::new(editor).arg(history.path.clone()).status();
    if !status.is_ok_and(|s| s.success()) {
        eprintln!("Error opening file in editor");
    } else {
        history.reload_content();
    }

    Ok(CommandResult::Continue)
}

fn sysprompt_command(command_params: CommandParams) -> io::Result<CommandResult> {
    command_params
        .chat_api
        .update_system_prompt(command_params.args.join(" "));
    Ok(CommandResult::Continue)
}

fn context_file_command(command_params: CommandParams) -> io::Result<CommandResult> {
    match command_params.args.first() {
        Some(new_context_file) => Ok(CommandResult::SwitchContext(Some(PathBuf::from(
            new_context_file,
        )))),
        _ => Ok(CommandResult::SwitchContext(None)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    struct MockApi {
        system_prompt: String,
    }

    impl MockApi {
        fn new() -> Self {
            Self {
                system_prompt: "".to_string(),
            }
        }
    }

    impl ChatApi for MockApi {
        fn generate_response(
            &self,
            _: serde_json::Value,
            _: &str,
            _: Option<&str>,
        ) -> std::io::Result<String> {
            Ok("Hello".to_string())
        }

        fn model_context_size(&self) -> Option<usize> {
            None
        }

        fn update_system_prompt(&mut self, system_prompt: String) {
            self.system_prompt = system_prompt;
        }
    }

    /// Helper function to create the test environment
    fn setup_test_environment() -> (Box<dyn ChatApi>, HistoryFile, TempDir, String) {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path().to_str().unwrap().to_string();

        let chat_api = Box::new(MockApi::new());

        // Create a temporary history file with some content
        let history_path = format!("{}/test-history.txt", dir_path);
        fs::write(&history_path, "Test conversation content").unwrap();

        let history = HistoryFile::new("test-history.txt".to_string(), dir_path.clone()).unwrap();

        (chat_api, history, temp_dir, dir_path)
    }

    #[test]
    fn test_list_command() -> io::Result<()> {
        let (mut ollama_client, mut history, _temp_dir, dir_path) = setup_test_environment();

        // Create a few test history files
        fs::write(format!("{}/history1.txt", dir_path), "Content 1")?;
        fs::write(format!("{}/history2.txt", dir_path), "Content 2")?;

        let params = CommandParams::new(vec![], &mut ollama_client, &mut history, dir_path);

        let result = list_command(params)?;
        assert!(matches!(result, CommandResult::Continue));

        // We can't easily test the stdout output here without mocking,
        // but the command should run without errors

        Ok(())
    }

    #[test]
    fn test_switch_command() -> io::Result<()> {
        let (mut ollama_client, mut history, _temp_dir, dir_path) = setup_test_environment();

        // Create a test history file to switch to
        let new_history_file = "new-history.txt";
        fs::write(
            format!("{}/{}", dir_path, new_history_file),
            "New history content",
        )?;

        let args = vec![new_history_file.to_string()];
        let params = CommandParams::new(args, &mut ollama_client, &mut history, dir_path);

        let result = switch_command(params)?;

        if let CommandResult::SwitchHistory(filename) = result {
            assert_eq!(filename, new_history_file);
        } else {
            panic!("Expected SwitchHistory result but got something else");
        }

        Ok(())
    }

    #[test]
    fn test_help_command() -> io::Result<()> {
        let (mut ollama_client, mut history, _temp_dir, dir_path) = setup_test_environment();

        let params = CommandParams::new(vec![], &mut ollama_client, &mut history, dir_path);

        let result = help_command(params)?;
        assert!(matches!(result, CommandResult::Continue));

        Ok(())
    }

    #[test]
    fn test_exit_command() -> io::Result<()> {
        let (mut ollama_client, mut history, _temp_dir, dir_path) = setup_test_environment();

        let params = CommandParams::new(vec![], &mut ollama_client, &mut history, dir_path);

        let result = quit_command(params)?;
        assert!(matches!(result, CommandResult::Quit));

        Ok(())
    }

    #[test]
    fn test_edit_command() -> io::Result<()> {
        let (mut ollama_client, mut history, _temp_dir, dir_path) = setup_test_environment();

        // We'll mock the editor by setting it to "echo" which should exist on most systems
        // and will just return successfully without doing anything
        unsafe {
            env::set_var("EDITOR", "echo");
        }

        let params = CommandParams::new(vec![], &mut ollama_client, &mut history, dir_path);

        let result = edit_command(params)?;
        assert!(matches!(result, CommandResult::Continue));

        Ok(())
    }

    #[test]
    fn test_sysprompt_command() -> io::Result<()> {
        let (mut ollama_client, mut history, _temp_dir, dir_path) = setup_test_environment();

        let test_prompt = "This is a test system prompt";
        let args: Vec<String> = test_prompt
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();
        let params = CommandParams::new(args, &mut ollama_client, &mut history, dir_path);

        let result = sysprompt_command(params)?;
        assert!(matches!(result, CommandResult::Continue));

        // Verify the prompt was updated
        // assert_eq!(ollama_client.system_prompt, test_prompt);

        Ok(())
    }

    #[test]
    fn test_create_command_registry() {
        let temp_map = HashMap::new();
        let registry = create_command_registry(temp_map);

        // Check that all expected command are registered
        assert!(registry.contains_key("q"));
        assert!(registry.contains_key("list"));
        assert!(registry.contains_key("switch"));
        assert!(registry.contains_key("sysprompt"));
        assert!(registry.contains_key("help"));
        assert!(registry.contains_key("edit"));
        assert!(registry.contains_key("context"));

        // Check the total number of command
        assert_eq!(registry.len(), 7);
    }

    #[test]
    fn test_switch_command_with_no_args() -> io::Result<()> {
        let (mut ollama_client, mut history, _temp_dir, dir_path) = setup_test_environment();

        let params = CommandParams::new(vec![], &mut ollama_client, &mut history, dir_path);

        let result = switch_command(params)?;
        assert!(matches!(result, CommandResult::Continue));

        Ok(())
    }

    #[test]
    fn test_list_command_with_pattern() -> io::Result<()> {
        let (mut ollama_client, mut history, _temp_dir, dir_path) = setup_test_environment();

        // Create some test files
        fs::write(format!("{}/history1.txt", dir_path), "Content 1")?;
        fs::write(format!("{}/history2.txt", dir_path), "Content 2")?;
        fs::write(format!("{}/other.txt", dir_path), "Other content")?;

        // Test with a pattern that should match some files
        let args = vec!["history".to_string()];
        let params = CommandParams::new(args, &mut ollama_client, &mut history, dir_path);

        let result = list_command(params)?;
        assert!(matches!(result, CommandResult::Continue));

        Ok(())
    }
}
