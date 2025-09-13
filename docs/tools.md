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

## Supported tools

### grep

Search for a pattern in the current directory with a two-second timeout,
1000 max matches and a max output size of 1 MiB.

Command:

```bash
grep -F --max-count=1000 <pattern> *
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
