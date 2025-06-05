# silent-llama
A command-line interface for interacting with Ollama AI models.

## Features
- Store conversations as files
- Add context to a session with `-f/--file` flag

## Installation
```shell
git clone https://github.com/yourusername/silent-llama.git
cd silent-llama
cargo build --release
```

## Dependencies
- clap
- crossterm
- Ollama

## Usage
```shell
sllama <HISTORY_FILE> [OPTIONS]
```

### Arguments
- `<HISTORY_FILE>` - Path to the file that acts as chat history (will be created if it doesn't exist)

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

## Configuration
You can configure your sllama

## License
[MIT License](LICENSE)
