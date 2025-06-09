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

use std::fs::OpenOptions;
use std::io;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

pub(crate) struct HistoryFile {
    pub(crate) path: String,
    pub(crate) filename: String,
    content: String,
}

impl HistoryFile {
    pub(crate) fn new(path: String, sllama_dir: String) -> io::Result<Self> {
        let full_path = if Path::new(&path).is_absolute() {
            println!("Opening file from absolute path: {}", path);
            PathBuf::from(path)
        } else {
            let actual_path = Path::new(&sllama_dir).join(path);
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

    /// Get the content of the history file
    pub(crate) fn get_content(&self) -> &str {
        &self.content
    }

    /// Append user input to the history file and update internal content
    pub(crate) fn append_user_input(&mut self, input: &str) -> io::Result<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .open(&self.path)?;

        let entry = format!("\n\n--- User Input ---\n\n{}", input);
        file.write_all(entry.as_bytes())?;

        self.content.push_str(&entry);

        Ok(())
    }

    /// Append AI response to the history file and update internal content
    pub(crate) fn append_ai_response(
        &mut self,
        response: &str,
        was_interrupted: bool,
    ) -> io::Result<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .open(&self.path)?;

        let response_with_note = if was_interrupted {
            format!("{}\n\n[Response was interrupted by user]", response)
        } else {
            response.to_string()
        };

        let entry = format!("\n\n--- AI Response ---\n\n{}", response_with_note);
        file.write_all(entry.as_bytes())?;

        self.content.push_str(&entry);

        Ok(())
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
        temp_path.close().unwrap(); // Delete the file

        let history_file = HistoryFile::new(path.clone(), String::new()).unwrap();

        assert!(fs::metadata(&path).is_ok()); // File exists
        assert_eq!(history_file.get_content(), ""); // Empty content
    }

    #[test]
    fn test_new_reads_existing_content() {
        let content = "Existing content";
        let temp_file = create_temp_file_with_content(content);
        let path = temp_file.path().to_str().unwrap().to_string();

        let history_file = HistoryFile::new(path, String::new()).unwrap();

        assert_eq!(history_file.get_content(), content);
    }

    #[test]
    fn test_append_user_input() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_str().unwrap().to_string();

        let mut history_file = HistoryFile::new(path.clone(), String::new()).unwrap();
        history_file.append_user_input("User message").unwrap();

        // Verify internal content was updated
        assert_eq!(
            history_file.get_content(),
            "\n\n--- User Input ---\n\nUser message"
        );

        // Verify file content was updated
        let file_content = fs::read_to_string(path).unwrap();
        assert_eq!(file_content, "\n\n--- User Input ---\n\nUser message");
    }

    #[test]
    fn test_append_multiple_entries() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_str().unwrap().to_string();

        let mut history_file = HistoryFile::new(path.clone(), String::new()).unwrap();
        history_file.append_user_input("User message 1").unwrap();
        history_file
            .append_ai_response("AI response 1", false)
            .unwrap();
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

        let mut history_file = HistoryFile::new(path.clone(), String::new()).unwrap();
        history_file
            .append_ai_response("AI response", false)
            .unwrap();

        // Verify internal content was updated
        assert_eq!(
            history_file.get_content(),
            "\n\n--- AI Response ---\n\nAI response"
        );

        // Verify file content was updated
        let file_content = fs::read_to_string(path).unwrap();
        assert_eq!(file_content, "\n\n--- AI Response ---\n\nAI response");
    }

    #[test]
    fn test_append_ai_response_interrupted() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_str().unwrap().to_string();

        let mut history_file = HistoryFile::new(path.clone(), String::new()).unwrap();
        history_file
            .append_ai_response("AI response", true)
            .unwrap();

        // Verify internal content was updated with interruption note
        assert!(
            history_file
                .get_content()
                .contains("[Response was interrupted by user]")
        );

        // Verify file content was updated with interruption note
        let file_content = fs::read_to_string(path).unwrap();
        assert!(file_content.contains("[Response was interrupted by user]"));
    }

    #[test]
    fn test_newline_handling() {
        let temp_file = create_temp_file_with_content("Initial content");
        let path = temp_file.path().to_str().unwrap().to_string();

        let mut history_file = HistoryFile::new(path, String::new()).unwrap();

        // First append doesn't need to add extra newline
        history_file.append_user_input("User input").unwrap();

        // Check that we don't have double newlines
        assert!(!history_file.get_content().contains("\n\n\n"));

        // Check that content is properly formatted
        assert!(
            history_file
                .get_content()
                .contains("Initial content\n\n--- User Input ---")
        );
    }

    #[test]
    fn test_history_file_with_relative_path() {
        // Create a temporary directory to act as sllama_dir
        let temp_dir = tempfile::tempdir().unwrap();
        let sllama_dir = temp_dir.path().to_string_lossy().to_string();

        // Use a relative path for the history file
        let relative_path = "test_history.txt".to_string();

        // Create the history file
        let history_file = HistoryFile::new(relative_path.clone(), sllama_dir.clone()).unwrap();

        // Expected full path (sllama_dir + relative_path)
        let expected_path = Path::new(&sllama_dir).join(&relative_path);

        // Verify the file exists at the expected path
        assert!(expected_path.exists());

        // Verify the path stored in the HistoryFile is correct
        assert_eq!(
            history_file.path,
            expected_path.to_string_lossy().to_string()
        );
    }

    #[test]
    fn test_history_file_with_absolute_path() {
        // Create a temporary directory to act as sllama_dir
        let temp_dir = tempfile::tempdir().unwrap();
        let sllama_dir = temp_dir.path().to_string_lossy().to_string();

        // Create another temporary directory for the absolute path
        let absolute_dir = tempfile::tempdir().unwrap();
        let absolute_path = absolute_dir
            .path()
            .join("absolute_history.txt")
            .to_string_lossy()
            .to_string();

        // Create the history file
        let history_file = HistoryFile::new(absolute_path.clone(), sllama_dir).unwrap();

        // Verify the file exists at the absolute path
        assert!(Path::new(&absolute_path).exists());

        // Verify the path stored in the HistoryFile is the absolute path
        assert_eq!(history_file.path, absolute_path);
    }
}
