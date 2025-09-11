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
use serde_json::Value;

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

pub fn get_tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "get_weather".to_string(),
            description: "Get weather for the user".to_string(),
            tool_fn: get_weather_tool,
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "city": {"type": "string"},
                    "country": {"type": "string"}
                },
                "required": ["city", "country"]
            })
        },
        Tool {
            name: "grep".to_string(),
            description: "Search for a pattern using 'grep' with *".to_string(),
            tool_fn: grep_tool,
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "pattern": {"type": "string"},
                },
                "required": ["pattern"]
            })
        },
        Tool {
            name: "pwd".to_string(),
            description: "Show current working directory".to_string(),
            tool_fn: pwd_tool,
            parameters: serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            })
        }
    ]
}

fn get_weather_tool(args: Value) -> String {
    format!(
        "The weather is snowy and -20 Celsius with a hint of chocolate in {}",
        args["city"]
    )
        .to_string()
}

fn grep_tool(args: Value) -> String {
    let pattern = args["pattern"].as_str().unwrap_or("");

    // Validate pattern for shell metacharacters
    if pattern.contains(|c: char| ";&|`$(){}[]<>\\\"'".contains(c)) {
        return "Error: Pattern contains invalid characters".to_string();
    }

    if pattern.is_empty() {
        return "Error: Empty pattern".to_string();
    }

    let output = std::process::Command::new("grep")
        .arg(pattern)
        .arg("*")
        .output()
        .expect("Failed to execute grep command");

    let result = String::from_utf8_lossy(&output.stdout).to_string();

    if result.is_empty() {
        "No matches found".to_string()
    } else {
        result.to_string()
    }
}

fn pwd_tool(_args: Value) -> String {
    let output = std::process::Command::new("pwd")
        .output()
        .expect("Failed to execute pwd command");

    String::from_utf8_lossy(&output.stdout).to_string()
}
