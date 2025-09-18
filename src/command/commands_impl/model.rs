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
use crate::config::profiles_config::ModelType;
use std::collections::HashMap;
use std::io;

pub(crate) fn new<'a>(_default_prefixes: &HashMap<String, String>) -> (String, CommandStruct<'a>) {
    (
        "model".to_string(),
        CommandStruct::new(
            "model",
            "Change current model type. Valid model types are 'fast', 'balanced', or 'deep'. If no model type is specified, the current model type is printed.",
            Some(":model <model_type>"),
            None,
            model_command,
            None,
        ),
    )
}

pub(crate) fn command<'a>(default_prefixes: &HashMap<String, String>) -> (String, CommandStruct<'a>) {
    new(default_prefixes)
}

pub(crate) fn model_command(command_params: CommandParams) -> io::Result<CommandResult> {
    match command_params.args.first() {
        Some(new_model) => {
            if let Ok(new_model) = ModelType::parse_model_type(new_model) {
                Ok(CommandResult::SwitchModel(new_model))
            } else {
                eprintln!(
                    "Error: Invalid model type specified: {}. Usage: :model <model>",
                    new_model
                );
                eprintln!("Valid models types are 'fast', 'balanced', or 'deep'\n");
                Ok(CommandResult::PrintModels)
            }
        }
        _ => Ok(CommandResult::PrintModels),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::setup_test_environment;
    use std::io;

    #[test]
    fn test_model_command_no_input() -> io::Result<()> {
        let (mut client, mut history, _tmp, dir_path) = setup_test_environment();
        let params = CommandParams::new(vec![], &mut client, &mut history, dir_path);
        let result = model_command(params)?;
        assert!(matches!(result, CommandResult::PrintModels));
        Ok(())
    }

    #[test]
    fn test_model_command_invalid_input() -> io::Result<()> {
        let (mut client, mut history, _tmp, dir_path) = setup_test_environment();
        let input = "not a valid model type";
        let args: Vec<String> = input.split_whitespace().map(|s| s.to_string()).collect();
        let params = CommandParams::new(args, &mut client, &mut history, dir_path);
        let result = model_command(params)?;
        assert!(matches!(result, CommandResult::PrintModels));
        Ok(())
    }

    #[test]
    fn test_model_command() -> io::Result<()> {
        let (mut client, mut history, _tmp, dir_path) = setup_test_environment();
        let args: Vec<String> = vec!["fast".to_string()];
        let params = CommandParams::new(args, &mut client, &mut history, dir_path);
        let result = model_command(params)?;
        assert!(matches!(result, CommandResult::SwitchModel(ModelType::Fast)));
        Ok(())
    }
}
