# Tools

If a model supports tools, they can decide to call them during the conversation.

**Note:** This is a stub of a feature that only has limited support with Ollama models and Linux.
More tools will be added in the future.

Tool calling is handled as described in
the [ollama API reference](https://deepwiki.com/ollama/ollama/3-api-reference "Link to ollama API reference").

All OS level tools are executed using the `std::process::Command` API.

## How it works

1. Check if the model has the `tools` capability
2. If it does, pass the tool definitions to the model
3. If the model decided to call a tool, it returns a `tool_call` object in the response
4. Loop through the tool calls
    1. Execute the tool call
    2. Pass the tool response to the model
    3. Display the model response to the user

## User tools

Users can define their own tools by adding one or more `.rs` files under `src/user_tools/`.
Each file must export a `pub fn tool() -> Tool` function returning a fully defined tool.

`src/user_tools/` is gitignored by default.

These tools are discovered at build time by [build.rs](../build.rs "Link to build.rs"),
which generates a `user_tools_gen.rs` module that is included in the build.
Changes under `src/user_tools/` trigger regeneration on rebuild.

### Quick start

1. Create `src/user_tools/hello.rs`.
2. Implement a `pub fn tool() -> Tool`.

```rust
use crate::config::AppConfig;
use crate::tool::tools::Tool;
use serde_json::Value;

pub fn tool() -> Tool {
    Tool::new(
        "hello",
        "Prints a greeting",
        serde_json::json!({
            "type": "object",
            "properties": { "name": { "type": "string" } },
            "required": ["name"]
        }),
        |args: Value, _cfg: Option<AppConfig>| -> String {
            let name = args.get("name").and_then(|v| v.as_str()).unwrap_or("world");
            format!("Hello, {name}!")
        },
    )
}
```

### Tool definition

```rust
pub struct Tool {
    pub(crate) name: String,
    pub(crate) description: String,
    tool_fn: ToolFn,
    pub parameters: Value,
}

impl Tool {
    pub fn execute(&self, args: Value, app_config: Option<AppConfig>) -> String {
        (self.tool_fn)(args, app_config)
    }

    pub fn new(name: &str, description: &str, parameters: Value, tool_fn: ToolFn) -> Self {
        Tool {
            name: name.to_string(),
            description: description.to_string(),
            tool_fn,
            parameters,
        }
    }

    pub fn json_definition(&self) -> Value {
        serde_json::json!({
            "type": "function",
            "function": {
                "name": self.name,
                "description": self.description,
                "parameters": self.parameters,
            }
        })
    }
}
```

## Built-in tools

### grep

Search for a pattern in the knowledge directory with 1000 max matches and a max output size of 1 MiB.

Command:

```bash
# current_dir set on the std::process:Command
grep -F -I -r --max-count=1000 <pattern>
```

Input uses the following allowlist:

- alphanumeric
- whitespace
- `-_.`

### pwd

Print the current working directory.

Command:

```bash
pwd
```

### Git Status

Show the status of the current git repository.

Command:

```bash
git status
```

### Git Diff

Show the diff of the current git repository.

Command:

```bash
git diff
```
