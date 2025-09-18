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
        "profile".to_string(),
        CommandStruct::new(
            "profile",
            "Change current profile by the profile name. If no profile is specified, the current profile is printed.",
            Some(":profile <profile>"),
            None,
            profile_command,
            None,
        ),
    )
}

pub(crate) fn command<'a>(default_prefixes: &HashMap<String, String>) -> (String, CommandStruct<'a>) {
    new(default_prefixes)
}

pub(crate) fn profile_command(command_params: CommandParams) -> io::Result<CommandResult> {
    match command_params.args.first() {
        Some(new_profile) => Ok(CommandResult::SwitchProfile(new_profile.to_string())),
        _ => Ok(CommandResult::PrintProfiles),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::setup_test_environment;
    use std::io;

    #[test]
    fn test_profile_command_no_input() -> io::Result<()> {
        let (mut client, mut history, _tmp, dir_path) = setup_test_environment();
        let params = CommandParams::new(vec![], &mut client, &mut history, dir_path);
        let result = profile_command(params)?;
        assert!(matches!(result, CommandResult::PrintProfiles));
        Ok(())
    }

    #[test]
    fn test_profile_command() -> io::Result<()> {
        let (mut client, mut history, _tmp, dir_path) = setup_test_environment();
        let args: Vec<String> = vec!["no_profile".to_string()];
        let params = CommandParams::new(args, &mut client, &mut history, dir_path);
        let result = profile_command(params)?;
        if let CommandResult::SwitchProfile(profile) = result {
            assert_eq!(profile, "no_profile");
        } else {
            panic!("Expected SwitchProfile result but got something else");
        }
        Ok(())
    }
}
