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
use crate::tool::tools::Tool;
use serde_json::Value;

pub fn tool() -> Tool {
    Tool::new(
        "grep",
        "Search for a pattern using 'grep' with *",
        serde_json::json!({
            "type": "object",
            "properties": {
                "pattern": {"type": "string"},
            },
            "required": ["pattern"]
        }),
        grep_impl,
    )
}
fn grep_impl(args: Value) -> String {
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
