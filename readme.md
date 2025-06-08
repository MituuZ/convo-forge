# silent-llama

A command-line interface for interacting with Ollama AI models.

## Features

- Store conversations as files
- Add context to a session with `-f/--file` flag
- Prompts are built in the following way:
    1. System prompt
    2. Context file
    3. History file
    4. Current user prompt

## Installation

```shell
git clone https://github.com/mituuz/silent-llama.git
cd silent-llama
cargo build --release
```

## Usage

```shell
sllama <HISTORY_FILE> [OPTIONS]
```

### Arguments

- `<HISTORY_FILE>` - Path to the file that acts as chat history (will be created if it doesn't exist)
    - If a relative path is provided, it will be created inside the `sllama_dir` directory
    - If an absolute path is provided, it will be used as-is regardless of `sllama_dir`

### Options

- `-f, --file <INPUT_FILE>` - Optional to be used as context for **each** chat message
- `-h, -help` - Print help
- `-v, --version` - Print version

### Example

```shell
# Start a new conversation saving history to chat.txt
sllama chat.txt

# Continue a conversation with additional context from code.rs
sllama chat.txt -f code.rs
```

### Commands

Commands can be entered during a chat by prepending the command with `:`. Commands are case-insensitive.

#### List

List all files in sllama_dir, optionally add a filter string.

`:list <filter>`

#### Switch

Switch to a different history file. Supports either absolute or relative paths (from `sllama_dir`).

`:switch relative/path`
`:switch /absolute/path`

#### Exit

Exit the current chat.

`:q`

#### Sysprompt

Update the system prompt for this session. Does not modify any configurations.

`:sysprompt Enter the new system prompt here`

## Configuration

You can configure your sllama by creating and modifying TOML configuration located at `~/.sllama.toml`/
`%USERPROFILE%\.sllama.toml`.

### Options

#### model

Ollama model used

Default: `gemma3:12b`

#### sllama_dir

Path to the sllama directory. This will hold new history files by default.

Default: `~/sllama`

#### system_prompt

System prompt that configures the AI assistant's behavior.

Default:

```
You are an AI assistant receiving input from a command-line
application called silent-llama (sllama). The user may include additional context from another file. 
This supplementary content appears after the system prompt and before the history file content.
Your responses are displayed in the terminal and saved to the history file.
Keep your answers helpful, concise, and relevant to both the user's direct query and any file context provided.
You can tell where you have previously responded by --- AI Response --- (added automatically).
```

## TODO

- [x] Clarify how the prompt is formed
- [x] Add a configuration file
- [x] Integrate rustyline
- [ ] Implement completions with rustyline
- [ ] Add functionality to truncate a chat
- [ ] Keep track of the model's context window and file size
- [ ] Create memories, which are included in the prompt by default (session/global)
- [ ] clap's arg groups can create mutually exclusive arguments

## Dependencies

- [Ollama](https://github.com/ollama/ollama) - [MIT](LICENSES/ollama-MIT)
- [serde](https://github.com/serde-rs/serde) - [MIT](LICENSES/serde-MIT)
- [toml](https://github.com/toml-rs/toml) - [MIT](LICENSES/toml-MIT)
- [clap](https://github.com/clap-rs/clap) - [MIT](LICENSES/clap-MIT)
- [crossterm](https://github.com/crossterm-rs/crossterm) - [MIT](LICENSES/crossterm-MIT)
- [tempfile](https://github.com/Stebalien/tempfile) - [MIT](LICENSES/tempfile-MIT)
- [rustyline](https://github.com/kkawakam/rustyline) - [MIT](LICENSES/rustyline-MIT)

## License

[MIT License](LICENSE)
