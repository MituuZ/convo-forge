# convo-forge

A command-line interface for interacting with Ollama and Anthropic models.

## Features

- Store conversations as files, allowing easy storage and editing
- Add context to a session with `-f/--file` flag and change the context file mid conversation
- Use commands to modify and customize the current session
- Newlines are supported with ALT + ENTER
- Reuse and modify prompts
- Define multiple profiles with up to three models per profile (fast, balanced, deep)
- Switch between profiles and models on the fly

How the messages array is formed in the request JSON:

| Role             | Content                                  |
|------------------|------------------------------------------|
| system/assistant | cforge system prompt                     |
| user/assistant   | conversation history                     |
| user             | current prompt (+ optional context file) |

[Wishlist at docs/todo.md](docs/todo.md)

## Quick start

```bash
# Note, requires the default model to be present for ollama (gemma3:12b)
git clone https://github.com/mituuz/convo-forge.git
cd convo-forge
# When running the command for the first time, it generates a config file with the default values
cargo run -- chat.md

# Basic commands
:help   # Show available commands
:list   # List chat files
:q      # Exit
```

## Requirements

- Rust (latest stable)
- Ollama or access to Anthropic API

## Installation

```bash
git clone https://github.com/mituuz/convo-forge.git
cd convo-forge
cargo build --release
```

cforge uses XDG paths for default chat and configuration.

## Usage

```bash
# First time / optional
cforge <HISTORY_FILE> [OPTIONS]

# After first time
cforge [OPTIONS]
```

### Arguments

- `<HISTORY_FILE>` - Path to the file that acts as chat history (will be created if it doesn't exist)
    - If a relative path is provided, it will be created inside the data directory (according to XDG)
    - If an absolute path is provided, it will be used as-is
    - Mandatory for the first time, after that `.cforge.toml` contains a reference to the previously opened history file

### Options

- `-f, --file <INPUT_FILE>` - Optional to be used as context for **each** chat message. Context file is reloaded with
  each message
- `-h, -help` - Print help
- `-v, --version` - Print version

### Example

```shell
# Start a new conversation saving history to chat.txt
cforge chat.txt

# Continue a conversation with additional context from code.rs
cforge chat.txt -f code.rs
```

### Commands

For a full list of commands, see [docs/commands.md](docs/commands.md).

## Configuration

You can configure your cforge by creating and modifying TOML configuration located at `~/.config/cforge/cforge.toml`.

An example toml populated with the default values.

```toml
# Path to the knowledge directory.
# Aliased to `@k/`
knowledge_dir = ""

# System prompt that configures the AI assistant's behavior.
system_prompt = """
You are an AI assistant receiving input from a command-line
application called convo-forge (cforge). The user may include additional context from another file,
this is included as a separate user prompt.
Your responses are displayed in the terminal and saved to the history file.
Keep your answers helpful, concise, and relevant to both the user's direct query and any file context provided.
"""

# Show estimated token count compared to the model's on each prompt if the provider supports it (ollama yes, anthropic no)
token_estimation = true

# Control the token limit for anthropic models
max_tokens = 1024

# Modify default prefixes for command completion
# Options support path aliases and absolute paths
# e.g. `:swi <tab> :switch @c/`
# e.g. `:swi <tab> :switch /home/user/my_dir`
[command_prefixes]
switch = "@c/"
list = "@c/"
context = "@k/"
prompt = "@p/"

# You can define multiple profiles with up to three model types per profile (fast, balanced, deep)
[profiles_config]
[[profiles_config.profiles]]
name = "local"
provider = "ollama"

[[profiles_config.profiles.models]]
model = "gemma3:12b"
model_type = "balanced"

[rustyline]
# Switch rustyline input mode between `emacs` and `vi`.
mode = "emacs"

# Switch completion type between `circular` and `list`.
completion_mode = "circular"
```

### Env variables

* **ANTHROPIC_API_KEY** - Valid API key to use Anthropic's models

## Security & Privacy

If you want to keep everything under your own control,
you should only use your local ollama. Nothing has to leave your machine.

Keep in mind that the chat files are stored on your machine and
there is no option for temporary chats.

Keep your API keys safe.

## Changelog

You can find the changelog [here](changelog.md "Link to changelog.md").

## Dependencies

- [Ollama](https://github.com/ollama/ollama) - [MIT](LICENSES/ollama-MIT)
- [serde](https://github.com/serde-rs/serde) - [MIT](LICENSES/serde-MIT)
- [serde_json](https://github.com/serde-rs/json) - [MIT](LICENSES/serde_json-MIT)
- [ureq](https://github.com/algesten/ureq) - [MIT](LICENSES/serde_json-MIT)
- [toml](https://github.com/toml-rs/toml) - [MIT](LICENSES/toml-MIT)
- [clap](https://github.com/clap-rs/clap) - [MIT](LICENSES/clap-MIT)
- [tempfile](https://github.com/Stebalien/tempfile) - [MIT](LICENSES/tempfile-MIT)
- [rustyline](https://github.com/kkawakam/rustyline) - [MIT](LICENSES/rustyline-MIT)
- [regex](https://github.com/rust-lang/regex) - [MIT](LICENSES/regex-MIT)
- [lazy-static](https://github.com/rust-lang-nursery/lazy-static.rs) - [MIT](LICENSES/lazy_static-MIT)
- [colored](https://github.com/colored-rs/colored) - [MPL-2.0](LICENSES/colored-MPL-2.0)
- [dirs-next](https://github.com/xdg-rs/dirs/tree/master/dirs) - [MIT](LICENSES/dirs-next-MIT)

## License

[MIT License](LICENSE)
