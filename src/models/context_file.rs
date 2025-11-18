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

use std::fs;
use std::path::PathBuf;

use crate::traits::estimate_context_size::ContextEstimation;

pub(crate) struct ContextFile {
    pub(crate) content: Option<String>,
}

impl ContextFile {
    pub(crate) fn new(context_file_path: &Option<PathBuf>) -> Self {
        // Read the context file if provided
        let content = if let Some(file_path) = &context_file_path {
            match fs::read_to_string(file_path.clone()) {
                Ok(content) => Some(content),
                Err(e) => {
                    eprintln!("Error reading context file: {e}");
                    None
                }
            }
        } else {
            None
        };

        ContextFile { content }
    }
}

impl ContextEstimation for ContextFile {
    fn estimate_context_size(&self) -> usize {
        let char_count = match &self.content {
            Some(c) => c.chars().count(),
            None => 0,
        };
        char_count / 4 + 1
    }
}
