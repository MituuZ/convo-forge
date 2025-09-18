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
use std::fs;
use std::io;

pub(crate) fn new<'a>(default_prefixes: &HashMap<String, String>) -> (String, CommandStruct<'a>) {
    (
        "list".to_string(),
        CommandStruct::new(
            "list",
            "List files in the cforge directory.                     Optionally, you can provide a pattern to filter the results.",
            Some(":list <optional pattern>"),
            Some(FileCommandDirectory::Cforge),
            list_command,
            default_prefixes.get("list").cloned(),
        ),
    )
}

pub(crate) fn command<'a>(default_prefixes: &HashMap<String, String>) -> (String, CommandStruct<'a>) {
    new(default_prefixes)
}

pub(crate) fn list_command(command_params: CommandParams) -> io::Result<CommandResult> {
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

    let cforge_dir = &command_params.cforge_dir.clone();
    list_dir_contents(cforge_dir, pattern, cforge_dir)?;

    Ok(CommandResult::Continue)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::setup_test_environment;
    use std::{fs, io};

    #[test]
    fn test_list_command() -> io::Result<()> {
        let (mut client, mut history, _tmp, dir_path) = setup_test_environment();
        fs::write(format!("{}/history1.txt", dir_path), "Content 1")?;
        fs::write(format!("{}/history2.txt", dir_path), "Content 2")?;
        let params = CommandParams::new(vec![], &mut client, &mut history, dir_path);
        let result = list_command(params)?;
        assert!(matches!(result, CommandResult::Continue));
        Ok(())
    }

    #[test]
    fn test_list_command_with_pattern() -> io::Result<()> {
        let (mut client, mut history, _tmp, dir_path) = setup_test_environment();
        fs::write(format!("{}/history1.txt", dir_path), "Content 1")?;
        fs::write(format!("{}/history2.txt", dir_path), "Content 2")?;
        fs::write(format!("{}/other.txt", dir_path), "Other content")?;
        let args = vec!["history".to_string()];
        let params = CommandParams::new(args, &mut client, &mut history, dir_path);
        let result = list_command(params)?;
        assert!(matches!(result, CommandResult::Continue));
        Ok(())
    }
}
