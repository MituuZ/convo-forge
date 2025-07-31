# convo-forge

A command-line interface for interacting with Ollama and Anthropic models.

## Features

- Store conversations as files, allowing easy storage and editing
- Add context to a session with `-f/--file` flag and change the context file mid conversation
- Use commands to modify and customize the current session
- Newlines are supported with ALT + ENTER
- Reuse and modify prompts

How the messages array is formed in the request JSON:

| Role             | Content                                  |
|------------------|------------------------------------------|
| system/assistant | cforge system prompt                     |
| user/assistant   | conversation history                     |
| user             | current prompt (+ optional context file) |

[Wishlist at docs/todo.md](docs/todo.md)

## Quick start

```bash
git clone https://github.com/mituuz/convo-forge.git
cd convo-forge
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

Commands can be entered during a chat by prepending the command with `:`. Commands are case-insensitive.

- [Help](#help)
- [List](#list)
- [Switch](#switch)
- [Edit](#edit)
- [Exit](#exit)
- [Sysprompt](#sysprompt)
- [Context](#context)

#### Path aliases

These can be used to quickly find files from cforge and knowledge directories without having to write the full path.

- `/` - Absolute path
- ` ` - Relative to the current dir
- `@c/` - Expands to the data directory
- `@k/` - Expands to the knowledge directory
- `@p/` - Expands to the prompt directory

You can [configure](#configuration) each file command with a custom prefix, either a path alias or absolute path.

e.g. `:swi <tab> :switch @c/`
e.g. `:swi <tab> :switch /home/user/my_dir`

#### Help

List available commands.

`:help`

#### List

List all files in the data directory, optionally add a filter string.

`:list <filter>`

#### Switch

Switch to a different history file. Supports either absolute or relative paths in the data directory.

`:switch relative/path`
`:switch /absolute/path`

Supports [path aliases](#path-aliases)

#### Edit

Open the current history file in the user's editor.

1. `$EDITOR`
2. `$VISUAL`
3. windows - `notepad` (untested)
4. other - `vi`

`:edit`

#### Exit

Exit the current chat.

`:q`

#### Sysprompt

Update the system prompt for this session. Does not modify any configurations.

`:sysprompt Enter the new system prompt here`

#### Context

Change the context file for this session.

`:context relative/path`
`:context /absolute/path`

Supports [path aliases](#path-aliases)

#### Prompt

Use or edit a prompt file. You can use `${{user_prompt}}` in a prompt file to control where
the user prompt is inserted when the message is sent, if not included,
the user prompt is appended after the prompt file.

To use a prompt file, write your actual prompt after the command and file.
(e.g., using ALT + ENTER to move to the next line)

```
:prompt /path/to/file
User prompt to send along the selected prompt
```

Edit a prompt by just calling

`:prompt relative/path`
`:prompt /absolute/path`

## Configuration

You can configure your cforge by creating and modifying TOML configuration located at `~/.config/cforge/cforge.toml`.

An example toml populated with the default values.

```toml
# AI model used.
model = "gemma3:12b"

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

# ollama/anthropic
# To use anthropic, use must have an environment variable `ANTHROPIC_API_KEY` set with a valid API key
provider = "ollama"

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
