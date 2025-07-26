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

    if !context_content.is_empty() {
        messages.push(serde_json::json!({ "role": "user", "content": format!("Additional context that should be considered: {}", context_content) }));
    }

    if let Some(history_messages_json) = history_messages_json.as_array() {
        for message in history_messages_json {
            messages.push(message.clone());
        }
    }

    messages.push(serde_json::json!({ "role": "user", "content": user_prompt }));

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

        assert_eq!(messages.len(), 3);
        assert_eq!(
            messages[0],
            json!({"role": "system", "content": "You are a helpful assistant."})
        );
        assert_eq!(
            messages[1],
            json!({"role": "user", "content": "Additional context that should be considered: This is some context."})
        );
        assert_eq!(messages[2], json!({"role": "user", "content": "Hello!"}));
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

        assert_eq!(messages.len(), 5);
        assert_eq!(
            messages[0],
            json!({"role": "system", "content": "You are a helpful assistant."})
        );
        assert_eq!(
            messages[1],
            json!({"role": "user", "content": "Additional context that should be considered: User is a developer."})
        );
        assert_eq!(messages[2], json!({"role": "user", "content": "Hello!"}));
        assert_eq!(
            messages[3],
            json!({"role": "assistant", "content": "Hi there! How can I help you today?"})
        );
        assert_eq!(
            messages[4],
            json!({"role": "user", "content": "Can you explain async/await?"})
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
