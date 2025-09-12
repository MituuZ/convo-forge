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
use crate::tools_impl;
use colored::Colorize;
use serde_json::Value;
use std::fmt::{Display, Formatter};

type ToolFn = fn(Value) -> String;

pub struct Tool {
    pub(crate) name: String,
    pub(crate) description: String,
    tool_fn: ToolFn,
    pub parameters: Value,
}

impl Tool {
    pub fn execute(&self, args: Value) -> String {
        (self.tool_fn)(args)
    }

    pub fn new(name: &str, description: &str, parameters: Value, tool_fn: ToolFn) -> Self {
        Tool {
            name: name.to_string(),
            description: description.to_string(),
            tool_fn,
            parameters,
        }
    }

    pub fn json_definition(&self) -> Value {
        serde_json::json!({
            "type": "function",
            "function": {
                "name": self.name,
                "description": self.description,
                "parameters": self.parameters,
            }
        })
    }
}

impl Display for Tool {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {}\n{} {}\n\n",
            "Name:".bold().cyan(),
            self.name,
            "Description:".bold().cyan(),
            self.description
        )
    }
}

pub fn get_tools() -> Vec<Tool> {
    vec![
        tools_impl::grep::tool(),
        tools_impl::pwd::tool(),
        tools_impl::git_status::tool(),
        tools_impl::git_diff::tool(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn test_tool_impl(args: Value) -> String {
        args.to_string()
    }

    fn get_test_tool() -> Tool {
        Tool::new(
            "Test Tools",
            "Used for testing",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "test_string": {"type": "string"},
                },
                "required": ["test_string"]
            }),
            test_tool_impl,
        )
    }

    #[test]
    fn test_tool_display() {
        let tool = get_test_tool();
        assert_eq!(
            format!("{}", tool),
            "Name: Test Tools\nDescription: Used for testing\n\n"
        );
    }

    #[test]
    fn test_tool_json_definition() {
        let tool = get_test_tool();
        assert_eq!(
            tool.json_definition(),
            serde_json::json!({
                "type": "function",
                "function": {
                    "name": "Test Tools",
                    "description": "Used for testing",
                    "parameters": serde_json::json!({
                        "type": "object",
                        "properties": {
                            "test_string": {"type": "string"},
                        },
                        "required": ["test_string"]
                    })
                }
            })
        )
    }

    #[test]
    fn test_tool_execution() {
        let tool = get_test_tool();
        assert_eq!(
            tool.execute(serde_json::json!({"test_string": "test"})),
            "{\"test_string\":\"test\"}"
        );
    }

    #[test]
    fn verify_tools_unique_names() {
        let tools = get_tools();
        let number_of_tools = tools.len();

        assert_eq!(tools.len(), number_of_tools);

        let mut seen_tool_names: HashSet<String> = HashSet::new();
        for tool in tools {
            assert!(
                seen_tool_names.insert(tool.name.clone()),
                "Tool name {} is not unique",
                tool.name
            );
        }

        assert_eq!(number_of_tools, seen_tool_names.len(), "Not all tools have unique names");
    }
}
