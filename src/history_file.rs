use std::fs::OpenOptions;
use std::io;
use std::io::{Read, Write};

pub(crate) struct HistoryFile {
    path: String,
    content: String,
}

impl HistoryFile {
    pub(crate) fn new(path: String) -> io::Result<Self> {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&path)?;

        // Read the current file content
        let mut content = String::new();
        file.read_to_string(&mut content)?;

        Ok(HistoryFile {
            path,
            content,
        })
    }
    
    /// Get the content of the history file
    pub(crate) fn get_content(&self) -> &str {
        &self.content
    }

    /// Append user input to the history file
    pub(crate) fn append_user_input(&mut self, input: &str) -> io::Result<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .open(&self.path)?;

        let entry = format!("\n\n--- User Input ---\n\n{}", input);
        file.write_all(entry.as_bytes())?;

        if !self.content.is_empty() && !self.content.ends_with("\n") {
            self.content.push('\n');
        }
        self.content.push_str(&entry);

        Ok(())
    }

    /// Append AI response to the history file
    pub(crate) fn append_ai_response(&mut self, response: &str, was_interrupted: bool) -> io::Result<()> {
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

        if !self.content.is_empty() && !self.content.ends_with("\n") {
            self.content.push('\n');
        }
        self.content.push_str(&entry);

        Ok(())
    }
}