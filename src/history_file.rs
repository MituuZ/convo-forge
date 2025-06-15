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
 *
 */

use lazy_static::lazy_static;
use regex::Regex;
use std::fs::OpenOptions;
use std::io;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

static DELIMITER_USER_INPUT: &str = r#"

-------------------------------------------------------------------
                        --- User Input ---
-------------------------------------------------------------------
"#;
static DELIMITER_AI_RESPONSE: &str = r#"

-------------------------------------------------------------------
                        --- AI Response ---
-------------------------------------------------------------------
"#;

lazy_static! {
    static ref DELIMITER_REGEX: Regex = {
        let pattern = format!(
            r"({}|{})",
            regex::escape(DELIMITER_USER_INPUT),
            regex::escape(DELIMITER_AI_RESPONSE)
        );
        Regex::new(&pattern).expect("Failed to compile regex pattern")
    };
}

#[derive(Debug)]
pub(crate) struct HistoryFile {
    pub(crate) path: String,
    pub(crate) filename: String,
    content: String,
}

impl HistoryFile {
    pub(crate) fn new(path: String, cforge_dir: String) -> io::Result<Self> {
        let full_path = if Path::new(&path).is_absolute() {
            println!("Opening file from absolute path: {}", path);
            PathBuf::from(path)
        } else {
            let actual_path = Path::new(&cforge_dir).join(path);
            let absolute_path =
                std::fs::canonicalize(&actual_path).unwrap_or_else(|_| actual_path.clone());
            println!(
                "Opening file from relative path: {}",
                absolute_path.display()
            );
            actual_path
        };

        let filename = full_path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned();

        if let Some(parent) = full_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let path_string = full_path.to_string_lossy().into_owned();

        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&full_path)?;

        // Read the current file content
        let mut content = String::new();
        file.read_to_string(&mut content)?;

        Ok(HistoryFile {
            path: path_string,
            content,
            filename,
        })
    }

    /// Get the content of the history file as a &str
    pub(crate) fn get_content(&self) -> &str {
        &self.content
    }

    /// Get the content of the history file formatted as a JSON array
    ///
    /// Returns a JSON array of `"role": "", "content": ""` messages
    pub(crate) fn get_content_json(&self) -> io::Result<serde_json::Value> {
        let mut messages = Vec::new();
        let mut matches_iter = DELIMITER_REGEX.find_iter(&self.content).peekable();

        if matches_iter.peek().is_none() {
            if let Some(message) = Self::maybe_create_message("user", &self.content) {
                messages.push(message);
            }
        } else {
            if let Some(first_match) = matches_iter.peek() {
                let start_position = first_match.start();
                if start_position > 0 {
                    let initial_text = &self.content[0..start_position];
                    if let Some(message) = Self::maybe_create_message("user", initial_text) {
                        messages.push(message);
                    }
                }
            }

            while let Some(current_match) = matches_iter.next() {
                let delimiter = &self.content[current_match.start()..current_match.end()];
                let role = if delimiter == DELIMITER_USER_INPUT {
                    "user"
                } else {
                    "assistant"
                };

                // Get the content after this delimiter but before the next
                let content_start = current_match.end();
                let content_end = matches_iter
                    .peek()
                    .map(|next_match| next_match.start())
                    .unwrap_or(self.content.len());

                if content_start < content_end {
                    let message_content = &self.content[content_start..content_end];
                    if let Some(message) = Self::maybe_create_message(role, message_content) {
                        messages.push(message);
                    }
                }
            }
        }

        Ok(serde_json::Value::Array(messages))
    }

    /// Tries to create a message from a role and content
    /// # Returns
    /// * `Some(Message)` if the content is not empty
    /// * `None` if the content is empty
    fn maybe_create_message(role: &str, content: &str) -> Option<serde_json::Value> {
        if content.trim().is_empty() {
            return None;
        }

        Some(serde_json::json!({
            "role": role,
            "content": content.trim()
        }))
    }

    /// Append user input to the history file and update internal content
    pub(crate) fn append_user_input(&mut self, input: &str) -> io::Result<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .open(&self.path)?;

        let entry = format!("{}{}", DELIMITER_USER_INPUT, input);
        file.write_all(entry.as_bytes())?;

        self.content.push_str(&entry);

        Ok(())
    }

    /// Append AI response to the history file and update internal content
    /// Return the response with the delimiter
    pub(crate) fn append_ai_response(&mut self, response: &str) -> io::Result<String> {
        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .open(&self.path)?;

        let response_with_note = response.to_string();

        let entry = format!("{}{}", DELIMITER_AI_RESPONSE, response_with_note);
        file.write_all(entry.as_bytes())?;

        self.content.push_str(&entry);

        Ok(entry)
    }

    pub(crate) fn reload_content(&mut self) {
        match OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(self.path.clone())
        {
            Ok(mut file) => {
                let mut content = String::new();
                file.read_to_string(&mut content).unwrap();
                self.content = content;
                println!("Reloaded file content: {}", self.path.clone());
            }
            Err(e) => println!("Error opening file: {}", e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_temp_file_with_content(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(content.as_bytes()).unwrap();
        file
    }

    #[test]
    fn test_new_creates_file_if_not_exists() {
        let temp_path = NamedTempFile::new().unwrap();
        let path = temp_path.path().to_str().unwrap().to_string();
        let expected_filename = temp_path
            .path()
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned();
        temp_path.close().unwrap(); // Delete the file

        let history_file = HistoryFile::new(path.clone(), String::new()).unwrap();

        assert!(fs::metadata(&path).is_ok()); // File exists
        assert_eq!(history_file.get_content(), ""); // Empty content
        assert_eq!(history_file.filename, expected_filename); // Filename is extracted correctly
    }

    #[test]
    fn test_new_reads_existing_content() {
        let content = "Existing content";
        let temp_file = create_temp_file_with_content(content);
        let path = temp_file.path().to_str().unwrap().to_string();
        let expected_filename = temp_file
            .path()
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned();

        let history_file = HistoryFile::new(path, String::new()).unwrap();

        assert_eq!(history_file.get_content(), content);
        assert_eq!(history_file.filename, expected_filename);
    }

    #[test]
    fn test_append_user_input() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_str().unwrap().to_string();
        let user_input = "User message";

        let mut history_file = HistoryFile::new(path.clone(), String::new()).unwrap();
        history_file.append_user_input(user_input).unwrap();

        let expected = format!("{}{}", DELIMITER_USER_INPUT, user_input);

        // Verify internal content was updated
        assert_eq!(history_file.get_content(), expected);

        // Verify file content was updated
        let file_content = fs::read_to_string(path).unwrap();
        assert_eq!(file_content, expected);
    }

    #[test]
    fn test_append_multiple_entries() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_str().unwrap().to_string();

        let mut history_file = HistoryFile::new(path.clone(), String::new()).unwrap();
        history_file.append_user_input("User message 1").unwrap();
        history_file.append_ai_response("AI response 1").unwrap();
        history_file.append_user_input("User message 2").unwrap();

        // Verify content has all entries
        let content = history_file.get_content();
        assert!(content.contains("User message 1"));
        assert!(content.contains("AI response 1"));
        assert!(content.contains("User message 2"));

        // Verify file content matches internal content
        let file_content = fs::read_to_string(path).unwrap();
        assert_eq!(file_content, content);
    }

    #[test]
    fn test_append_ai_response_normal() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_str().unwrap().to_string();
        let ai_response = "AI response";

        let mut history_file = HistoryFile::new(path.clone(), String::new()).unwrap();
        history_file.append_ai_response(ai_response).unwrap();

        let expected = format!("{}{}", DELIMITER_AI_RESPONSE, ai_response);

        // Verify internal content was updated
        assert_eq!(history_file.get_content(), expected);

        // Verify file content was updated
        let file_content = fs::read_to_string(path).unwrap();
        assert_eq!(file_content, expected);
    }

    #[test]
    fn test_newline_handling() {
        let temp_file = create_temp_file_with_content("Initial content");
        let path = temp_file.path().to_str().unwrap().to_string();
        let user_input = "User message";

        let mut history_file = HistoryFile::new(path, String::new()).unwrap();

        // First append doesn't need to add extra newline
        history_file.append_user_input(user_input).unwrap();

        // Check that we don't have double newlines
        assert!(!history_file.get_content().contains("\n\n\n"));

        let expected = format!("{}{}", DELIMITER_USER_INPUT, user_input);

        // Check that content is properly formatted
        assert!(history_file.get_content().contains(&expected));
    }

    #[test]
    fn test_history_file_with_relative_path() {
        let temp_dir = tempfile::tempdir().unwrap();
        let cforge_dir = temp_dir.path().to_string_lossy().to_string();

        // Use a relative path for the history file
        let relative_path = "test_history.txt".to_string();

        let history_file = HistoryFile::new(relative_path.clone(), cforge_dir.clone()).unwrap();
        let expected_path = Path::new(&cforge_dir).join(&relative_path);

        assert!(expected_path.exists());

        assert_eq!(
            history_file.path,
            expected_path.to_string_lossy().to_string()
        );

        assert_eq!(history_file.filename, "test_history.txt");
    }

    #[test]
    fn test_history_file_with_absolute_path() {
        let temp_dir = tempfile::tempdir().unwrap();
        let cforge_dir = temp_dir.path().to_string_lossy().to_string();

        // Create a temporary directory for the absolute path
        let absolute_dir = tempfile::tempdir().unwrap();
        let absolute_path = absolute_dir
            .path()
            .join("absolute_history.txt")
            .to_string_lossy()
            .to_string();

        // Create the history file
        let history_file = HistoryFile::new(absolute_path.clone(), cforge_dir).unwrap();

        // Verify the file exists at the absolute path
        assert!(Path::new(&absolute_path).exists());

        // Verify the path stored in the HistoryFile is the absolute path
        assert_eq!(history_file.path, absolute_path);

        // Verify the filename is extracted correctly
        assert_eq!(history_file.filename, "absolute_history.txt");
    }

    #[test]
    fn test_directory_path_handling() {
        let temp_dir = tempfile::tempdir().unwrap();
        let dir_path = temp_dir.path().to_string_lossy().to_string();

        // Attempt to create a history file with a directory path
        let result = HistoryFile::new(dir_path, String::new());

        // Should result in an error, not a panic
        assert!(result.is_err());

        // Just check that we get an error, without asserting on the specific error kind
        // since it can vary between operating systems
        let _error = result.unwrap_err();
        println!("Got expected error when opening directory: {:?}", _error);
    }

    #[test]
    fn test_json_parsing_empty_file() {
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().join("test.txt");
        let cforge_dir = temp_dir.path().to_string_lossy().to_string();
        let content = "";

        // Use a relative path for the history file
        let relative_path = "test_history.txt".to_string();
        let mut history_file = HistoryFile::new(relative_path.clone(), cforge_dir.clone()).unwrap();

        history_file.content = content.to_string();

        assert!(history_file.get_content_json().unwrap().is_array());
        assert!(
            history_file
                .get_content_json()
                .unwrap()
                .as_array()
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn test_json_parsing_linear_messages() {
        let temp_dir = tempfile::tempdir().unwrap();
        let cforge_dir = temp_dir.path().to_string_lossy().to_string();

        let content = format!(
            "{}{}{}{}{}{}",
            create_message(DELIMITER_USER_INPUT, "User message 1"),
            create_message(DELIMITER_AI_RESPONSE, "AI response 1"),
            create_message(DELIMITER_USER_INPUT, "User message 2"),
            create_message(DELIMITER_AI_RESPONSE, "AI response 2"),
            create_message(DELIMITER_USER_INPUT, "User message 3"),
            create_message(DELIMITER_AI_RESPONSE, "AI response 3"),
        );

        // Use a relative path for the history file
        let relative_path = "test_history.txt".to_string();
        let mut history_file = HistoryFile::new(relative_path.clone(), cforge_dir.clone()).unwrap();

        history_file.content = content;

        let expected = serde_json::json!([
                {
                    "role": "user",
                    "content": "User message 1"
                },
                {
                    "role": "assistant",
                    "content": "AI response 1"
                },
                {
                    "role": "user",
                    "content": "User message 2"
                },
                {
                    "role": "assistant",
                    "content": "AI response 2"
                },
                {
                    "role": "user",
                    "content": "User message 3"
                },
                {
                    "role": "assistant",
                    "content": "AI response 3"
                }
            ]
        );
        assert_eq!(history_file.get_content_json().unwrap(), expected);
    }

    #[test]
    fn test_json_parsing_non_linear_messages() {
        let temp_dir = tempfile::tempdir().unwrap();
        let cforge_dir = temp_dir.path().to_string_lossy().to_string();

        let content = format!(
            "{}{}{}{}{}{}",
            create_message(DELIMITER_USER_INPUT, "User message 1"),
            create_message(DELIMITER_USER_INPUT, "User message 2"),
            create_message(DELIMITER_AI_RESPONSE, "AI response 1"),
            create_message(DELIMITER_USER_INPUT, "User message 3"),
            create_message(DELIMITER_AI_RESPONSE, "AI response 2"),
            create_message(DELIMITER_AI_RESPONSE, "AI response 3"),
        );

        // Use a relative path for the history file
        let relative_path = "test_history.txt".to_string();
        let mut history_file = HistoryFile::new(relative_path.clone(), cforge_dir.clone()).unwrap();

        history_file.content = content;

        let expected = serde_json::json!([
                {
                    "role": "user",
                    "content": "User message 1"
                },
                {
                    "role": "user",
                    "content": "User message 2"
                },
                {
                    "role": "assistant",
                    "content": "AI response 1"
                },
                {
                    "role": "user",
                    "content": "User message 3"
                },
                {
                    "role": "assistant",
                    "content": "AI response 2"
                },
                {
                    "role": "assistant",
                    "content": "AI response 3"
                }
            ]
        );
        assert_eq!(history_file.get_content_json().unwrap(), expected);
    }

    #[test]
    fn test_json_parsing_empty_messages() {
        let temp_dir = tempfile::tempdir().unwrap();
        let cforge_dir = temp_dir.path().to_string_lossy().to_string();

        let content = format!(
            "{}{}{}",
            DELIMITER_USER_INPUT, DELIMITER_AI_RESPONSE, DELIMITER_USER_INPUT
        );

        let relative_path = "test_history.txt".to_string();
        let mut history_file = HistoryFile::new(relative_path, cforge_dir).unwrap();
        history_file.content = content;

        assert!(history_file.get_content_json().unwrap().is_array());
        assert!(
            history_file
                .get_content_json()
                .unwrap()
                .as_array()
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn test_json_parsing_whitespace_only_messages() {
        let temp_dir = tempfile::tempdir().unwrap();
        let cforge_dir = temp_dir.path().to_string_lossy().to_string();

        let content = format!(
            "{}{}{}{}",
            DELIMITER_USER_INPUT, "   \n  \t  ", DELIMITER_AI_RESPONSE, "  \n\n  "
        );

        let relative_path = "test_history.txt".to_string();
        let mut history_file = HistoryFile::new(relative_path, cforge_dir).unwrap();
        history_file.content = content;

        assert!(history_file.get_content_json().unwrap().is_array());
        assert!(
            history_file
                .get_content_json()
                .unwrap()
                .as_array()
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn test_json_parsing_content_without_delimiters() {
        let temp_dir = tempfile::tempdir().unwrap();
        let cforge_dir = temp_dir.path().to_string_lossy().to_string();

        let content = "This is some content without any delimiters.".to_string();

        let relative_path = "test_history.txt".to_string();
        let mut history_file = HistoryFile::new(relative_path, cforge_dir).unwrap();
        history_file.content = content.clone();

        assert!(history_file.get_content_json().unwrap().is_array());
        let expected = serde_json::json!([
                {
                    "role": "user",
                    "content": content
                }
            ]
        );
        assert_eq!(history_file.get_content_json().unwrap(), expected);
    }

    #[test]
    fn test_json_parsing_multiline_content() {
        let temp_dir = tempfile::tempdir().unwrap();
        let cforge_dir = temp_dir.path().to_string_lossy().to_string();

        let content = format!(
            "{}{}{}{}",
            DELIMITER_USER_INPUT,
            "Line 1\nLine 2\nLine 3",
            DELIMITER_AI_RESPONSE,
            "Response\nWith\nMultiple\nLines"
        );

        let relative_path = "test_history.txt".to_string();
        let mut history_file = HistoryFile::new(relative_path, cforge_dir).unwrap();
        history_file.content = content;

        let expected = serde_json::json!([
                {
                    "role": "user",
                    "content": "Line 1\nLine 2\nLine 3"
                },
                {
                    "role": "assistant",
                    "content": "Response\nWith\nMultiple\nLines"
                }
            ]
        );
        assert_eq!(history_file.get_content_json().unwrap(), expected);
    }

    #[test]
    fn test_json_parsing_with_large_content() {
        let temp_dir = tempfile::tempdir().unwrap();
        let cforge_dir = temp_dir.path().to_string_lossy().to_string();

        let large_text = "A".repeat(10_000);
        let content = format!("{}{}", DELIMITER_USER_INPUT, large_text);

        let relative_path = "test_history.txt".to_string();
        let mut history_file = HistoryFile::new(relative_path, cforge_dir).unwrap();
        history_file.content = content;

        let expected = serde_json::json!([
                {
                    "role": "user",
                    "content": large_text.trim()
                }
            ]
        );
        assert_eq!(history_file.get_content_json().unwrap(), expected);
    }

    #[test]
    fn test_maybe_create_message_with_empty_content() {
        let result = HistoryFile::maybe_create_message("user", "");
        assert_eq!(result, None);

        let result = HistoryFile::maybe_create_message("user", "  \n  \t  ");
        assert_eq!(result, None);
    }

    #[test]
    fn test_maybe_create_message_with_valid_content() {
        let result = HistoryFile::maybe_create_message("user", "Hello");
        let expected = Some(serde_json::json!({
            "role": "user",
            "content": "Hello"
        }));
        assert_eq!(result, expected);

        let result = HistoryFile::maybe_create_message("assistant", "  Response  ");
        let expected = Some(serde_json::json!({
            "role": "assistant",
            "content": "Response"
        }));
        assert_eq!(result, expected);
    }

    fn create_message(delimiter: &str, content: &str) -> String {
        format!("{}{}", delimiter, content)
    }
}
