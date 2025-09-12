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

pub(crate) fn create_messages(
    system_prompt: &str,
    context_content: &str,
    user_prompt: &str,
    history_messages_json: &Value,
    system_prompt_role: &str,
) -> Vec<Value> {
    let mut messages = vec![];

    messages.push(serde_json::json!({ "role": system_prompt_role, "content": system_prompt }));

    if let Some(history_messages_json) = history_messages_json.as_array() {
        for message in history_messages_json {
            messages.push(message.clone());
        }
    }

    let user_message = if context_content.is_empty() {
        user_prompt.to_string()
    } else {
        format!("{user_prompt}\n\nAdditional context: {context_content}")
    };
    messages.push(serde_json::json!({ "role": "user", "content": user_message }));

    messages
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::api::client_util::create_messages;

    #[test]
    fn test_create_messages_assistant() {
        let system_prompt = "You are a helpful assistant.";
        let context_content = "";
        let user_prompt = "Hello!";
        let history = json!([]);

        let messages = create_messages(
            system_prompt,
            context_content,
            user_prompt,
            &history,
            "assistant",
        );

        assert_eq!(messages.len(), 2);
        assert_eq!(
            messages[0],
            json!({"role": "assistant", "content": "You are a helpful assistant."})
        );
        assert_eq!(messages[1], json!({"role": "user", "content": "Hello!"}));
    }

    #[test]
    fn test_create_messages_system() {
        let system_prompt = "You are a helpful assistant.";
        let context_content = "";
        let user_prompt = "Hello!";
        let history = json!([]);

        let messages = create_messages(
            system_prompt,
            context_content,
            user_prompt,
            &history,
            "system",
        );

        assert_eq!(messages.len(), 2);
        assert_eq!(
            messages[0],
            json!({"role": "system", "content": "You are a helpful assistant."})
        );
        assert_eq!(messages[1], json!({"role": "user", "content": "Hello!"}));
    }

    #[test]
    fn test_create_messages_with_context() {
        let system_prompt = "You are a helpful assistant.";
        let context_content = "This is some context.";
        let user_prompt = "Hello!";
        let history = json!([]);

        let messages = create_messages(
            system_prompt,
            context_content,
            user_prompt,
            &history,
            "system",
        );

        assert_eq!(messages.len(), 2);
        assert_eq!(
            messages[0],
            json!({"role": "system", "content": "You are a helpful assistant."})
        );
        assert_eq!(
            messages[1],
            json!({"role": "user", "content": "Hello!\n\nAdditional context: This is some context."})
        );
    }

    #[test]
    fn test_create_messages_with_history() {
        let system_prompt = "You are a helpful assistant.";
        let context_content = "";
        let user_prompt = "How are you?";
        let history = json!([
            {"role": "user", "content": "Hello!"},
            {"role": "assistant", "content": "Hi there! How can I help you today?"}
        ]);

        let messages = create_messages(
            system_prompt,
            context_content,
            user_prompt,
            &history,
            "system",
        );

        assert_eq!(messages.len(), 4);
        assert_eq!(
            messages[0],
            json!({"role": "system", "content": "You are a helpful assistant."})
        );
        assert_eq!(messages[1], json!({"role": "user", "content": "Hello!"}));
        assert_eq!(
            messages[2],
            json!({"role": "assistant", "content": "Hi there! How can I help you today?"})
        );
        assert_eq!(
            messages[3],
            json!({"role": "user", "content": "How are you?"})
        );
    }

    #[test]
    fn test_create_messages_with_context_and_history() {
        let system_prompt = "You are a helpful assistant.";
        let context_content = "User is a developer.";
        let user_prompt = "Can you explain async/await?";
        let history = json!([
            {"role": "user", "content": "Hello!"},
            {"role": "assistant", "content": "Hi there! How can I help you today?"}
        ]);

        let messages = create_messages(
            system_prompt,
            context_content,
            user_prompt,
            &history,
            "system",
        );

        assert_eq!(messages.len(), 4);
        assert_eq!(
            messages[0],
            json!({"role": "system", "content": "You are a helpful assistant."})
        );
        assert_eq!(messages[1], json!({"role": "user", "content": "Hello!"}));
        assert_eq!(
            messages[2],
            json!({"role": "assistant", "content": "Hi there! How can I help you today?"})
        );
        assert_eq!(
            messages[3],
            json!({"role": "user", "content": "Can you explain async/await?\n\nAdditional context: User is a developer."})
        );
    }

    #[test]
    fn test_create_messages_with_invalid_history() {
        let system_prompt = "You are a helpful assistant.";
        let context_content = "";
        let user_prompt = "Hello!";
        let history = json!({"invalid": "not an array"}); // Not an array

        let messages = create_messages(
            system_prompt,
            context_content,
            user_prompt,
            &history,
            "system",
        );

        assert_eq!(messages.len(), 2);
        assert_eq!(
            messages[0],
            json!({"role": "system", "content": "You are a helpful assistant."})
        );
        assert_eq!(messages[1], json!({"role": "user", "content": "Hello!"}));
    }

    #[test]
    fn test_create_messages_with_empty_system_prompt() {
        let system_prompt = "";
        let context_content = "";
        let user_prompt = "Hello!";
        let history = json!([]);

        let messages = create_messages(
            system_prompt,
            context_content,
            user_prompt,
            &history,
            "system",
        );

        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0], json!({"role": "system", "content": ""}));
        assert_eq!(messages[1], json!({"role": "user", "content": "Hello!"}));
    }
}
