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
use crate::config::AppConfig;
use crate::tool::tools::Tool;
use serde_json::Value;

pub fn tool() -> Tool {
    Tool::new(
        "grep",
        "Search for a pattern using 'grep' from the knowledge dir\
        \nCommand: `grep -F --max-count=1000 <pattern> *`",
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

fn grep_impl(args: Value, app_config: Option<AppConfig>) -> String {
    let pattern = match args.get("pattern").and_then(|v| v.as_str()) {
        Some(p) => {
            if p.is_empty() {
                return "Error: Empty pattern".to_string();
            }
            p.to_string()
        }
        None => {
            return "Error: Missing pattern".to_string();
        }
    };

    let knowledge_base_path = match app_config {
        None => {
            return "Error: App config not found".to_string();
        }
        Some(app_config) => {
            app_config.user_config.knowledge_dir.clone()
        }
    };

    if knowledge_base_path.is_empty() {
        return "Error: Knowledge dir path is empty".to_string();
    }

    let canon = match std::fs::canonicalize(knowledge_base_path.clone()) {
        Ok(p) => p,
        Err(_) => {
            return format!(
                "Error: '{}' cannot be resolved to a real directory",
                knowledge_base_path
            );
        }
    };

    if !canon.is_dir() {
        return format!("Error: '{}' is not a directory", canon.display());
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
        .arg("-F")
        .arg("-I")
        .arg("-r")
        .arg("--max-count=1000")
        .arg(pattern.clone())
        .current_dir(canon.clone())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
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

    let result = String::from_utf8_lossy(&output.stdout)
        .trim_end()
        .to_string();

    if !output.status.success() {
        if output.status.code() == Some(1) && result.trim().is_empty() {
            return "No matches found".to_string();
        }

        return format!(
            "Error: `grep` failed (code {:?})\nMessage: {}",
            output.status.code(),
            result
        );
    }

    const MAX_BYTES: usize = 1_048_576; // 1 MiB
    if output.stdout.len() > MAX_BYTES {
        return "Error: Output exceeds size limit".into();
    }

    eprintln!(
        "[grep] dir='{}' pattern='{}' result='{}'",
        canon.display(),
        pattern,
        if result.is_empty() { "none" } else { "found" }
    );

    if result.is_empty() {
        "No matches found".to_string()
    } else {
        result.to_string()
    }
}
