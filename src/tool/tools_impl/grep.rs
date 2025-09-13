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
        "Search for a pattern using 'grep'\nCommand: `grep -F --max-count=1000 <pattern> *`",
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

    if pattern.is_empty() {
        return "Error: Empty pattern".to_string();
    }

    if !pattern
        .chars()
        .all(|c| c.is_alphanumeric() || c.is_whitespace() || c == '-' || c == '_' || c == '.')
    {
        return "Error: Pattern contains characters outside of the allowlist:\
\n- alphanumeric\
\n- whitespace\
\n- -_.
        "
            .to_string();
    }

    let mut child = match std::process::Command::new("grep")
        .arg(pattern)
        .arg("-F")
        .arg("-I")
        .arg("--max-count=1000")
        .arg("*")
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            return format!("Error launching grep: {}", e);
        }
    };

    let timeout = std::time::Duration::from_secs(3);
    let start = std::time::Instant::now();
    while start.elapsed() < timeout {
        if let Ok(Some(_)) = child.try_wait() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    if start.elapsed() >= timeout {
        let _ = child.kill();
        let _ = child.wait();
        return "Error: `grep` timed out".to_string();
    }

    let output = match child.wait_with_output() {
        Ok(output) => output,
        Err(error) => {
            return format!("Error collecting `grep` output: {}", error);
        }
    };

    let result = String::from_utf8_lossy(&output.stdout).to_string();

    if !output.status.success() {
        return format!(
            "Error: `grep` returned non-zero exit code: {}\nMessage: {}",
            output.status, result
        );
    }

    const MAX_BYTES: usize = 1_048_576; // 1 MiB
    if output.stdout.len() > MAX_BYTES {
        return "Error: Output exceeds size limit".into();
    }

    if result.is_empty() {
        "No matches found".to_string()
    } else {
        result.to_string()
    }
}
