# convo-forge

A command-line interface for interacting with Ollama API.

## Features

- Store conversations as files
- Add context to a session with `-f/--file` flag
- Use commands to modify and customize the current session
- Prompts are built in the following way and sent using the
  `/chat` [endpoint (without streaming)](https://github.com/ollama/ollama/blob/main/docs/api.md#chat-request-no-streaming)
- You can add a newline using ALT + ENTER

How the messages array is formed in the request JSON:

| Role           | Content                 |
|----------------|-------------------------|
| system         | cforge system prompt    |
| system         | context file (optional) |
| user/assistant | conversation history    |
| user           | current prompt          |

[Wishlist at docs/todo.md](docs/todo.md)

## Installation

```shell
git clone https://github.com/mituuz/convo-forge.git
cd convo-forge
cargo build --release
```

## Usage

```shell
# First time / optional
cforge <HISTORY_FILE> [OPTIONS]

# After first time
cforge [OPTIONS]
```

### Arguments

- `<HISTORY_FILE>` - Path to the file that acts as chat history (will be created if it doesn't exist)
    - If a relative path is provided, it will be created inside the `cforge_dir` directory
    - If an absolute path is provided, it will be used as-is regardless of `cforge_dir`
    - Mandatory for the first time, after that `.cforge.toml` contains a reference to the previously opened history file

### Options

- `-f, --file <INPUT_FILE>` - Optional to be used as context for **each** chat message
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

#### Help

List available commands.

`:help`

#### List

List all files in cforge_dir, optionally add a filter string.

`:list <filter>`

#### Switch

Switch to a different history file. Supports either absolute or relative paths (from `cforge_dir`).

`:switch relative/path`
`:switch /absolute/path`

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

## Configuration

You can configure your cforge by creating and modifying TOML configuration located at `~/.cforge.toml`/
`%USERPROFILE%\.cforge.toml`.

An example toml populated with the default values.

```toml
# Ollama model used.
model = "gemma3:12b"

# Path to the cforge directory. This will hold new history files by default.
# ~ is expanded to the user's home directory based on `$HOME` or `%USERPROFILE%`. (not verified on windows)
cforge_dir = "~/cforge"

# System prompt that configures the AI assistant's behavior.
system_prompt = """
You are an AI assistant receiving input from a command-line
application called convo-forge (cforge). The user may include additional context from another file,
this is included as a system prompt.
Your responses are displayed in the terminal and saved to the history file.
Keep your answers helpful, concise, and relevant to both the user's direct query and any file context provided.
"""

# Show estimated token count compared to the model's on each prompt
token_estimation = true

[rustyline]
# Switch rustyline input mode between `emacs` and `vi`.
mode = "emacs"

# Switch completion type between `circular` and `list`.
completion_mode = "circular"
```

### Configuring Ollama

Ollama unloads the models after a set time. This can be controlled either from an environment variable or through the
[message itself](docs/todo.md).

cforge sends an empty message to preload the model before calling it and tries to resend messages that get an empty
response from the model.

[Ollama Docs - Keeping a model loaded in memory](https://ollama.readthedocs.io/en/faq/?h=keep#how-do-i-keep-a-model-loaded-in-memory-or-make-it-unload-immediately)

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
- [colored](https://github.com/colored-rs/colored) - [MPL-2.0]()

## License

[MIT License](LICENSE)
