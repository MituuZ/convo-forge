# Changelog

## 0.2.0 - 2025-06-15

_Parse history file to JSON array of messages using `user` and `assistant` roles_

### "Breaking" change

Because the delimiters have changed for better visibility and parsing. Messages that do not adhere to this format are
sent as a single user message.

## 0.1.13 - 2025-06-14

_Rename project to `convo-forge` (`cforge`)_

### Why

Renamed from the previous name to better reflect the project's purpose and to avoid being tied specifically to Ollama,
as adding API support paves the way for supporting multiple LLM providers.

### Changes

- Updated package name in `Cargo.toml`
- Updated binary name from `cforge`
- Updated internal and file system references

## 0.1.12 - 2025-06-14

_Add support for rustyline completion type (circular and list)_

## 0.1.11 - 2025-06-14

_Switch to Ollama API instead of run commands_

### Prompting

Update the message format to match the `/chat` endpoint requirements.

## 0.1.10 - 2025-06-14

_Implement basic command completion and hinting with rustyline_

## 0.1.9 - 2025-06-12

_Add support for rustyline modes_

## 0.1.8 - 2025-06-09

_Refactor command handling and remove crossterm_

## 0.1.7 - 2025-06-08

_Add `edit` command_

### Commands

- Enable modifying the currently open history file
    1. `$EDITOR`
    2. `$VISUAL`
    3. windows - `notepad`
    4. other - `vi`

## 0.1.6 - 2025-06-08

_Add `help` command_

## 0.1.5 - 2025-06-08

_Integrate [rustyline](https://github.com/kkawakam/rustyline) for input handling, add `sysprompt` command, order prompts
and disable interruption_

### Interruption

- Crossterm and rustyline do not play nice, disabled AI response interruption to keep input consistent.

### Prompts

- Use a more traditional ordering with:
    1. System prompt
    2. Context file
    3. History file
    4. Current user prompt

### Commands

- Enable modifying system prompt for the current session.

## 0.1.4 - 2025-06-08

_Add `switch` command_

### Commands

- Enables changing the history file during an active chat

## 0.1.3 - 2025-06-07

_Add `list` command_

### Commands

- First active chat command `:list`, list files from your cforge_dir with an optional parameter to filter
- Modify exit to a command `:q`

## 0.1.2 - 2025-06-06

_Implement history file_

### Configs

- `cforge_dir` - Now works as expected. Relative files are created here, but absolute paths are respected

## 0.1.1 - 2025-06-06

_Add configuration and changelog files_

### Configs

- `cforge_dir` - Not applied yet, will be default location for history files
- `model` - Switch used model
- `system_prompt` - Customize the prompt that cforge provides to the model

## 0.1.0 - 2025-06-05

_Initial release_

### Features

- Actual files as chat history
- Add context from a file to each chat message
