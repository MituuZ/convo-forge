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
use std::env::var;

/// Attempts to determine the user's preferred text editor by checking
/// environment variables, falling back to a default editor based on the OS.
///
/// # Procedure
/// 1. Checks the `EDITOR` environment variable for a value.
/// 2. If `EDITOR` is not set, checks the `VISUAL` environment variable.
/// 3. If both `EDITOR` and `VISUAL` are not set:
///    - Defaults to "notepad" on Windows.
///    - Defaults to "vi" on all other platforms.
///
/// # Returns
/// A `String` representing the fully resolved editor name.
pub(crate) fn get_editor() -> String {
    var("EDITOR")
        .or_else(|_| var("VISUAL"))
        .unwrap_or_else(|_| {
            if cfg!(target_os = "windows") {
                "notepad".to_string()
            } else {
                "vi".to_string()
            }
        })
}
