# Changelog

## 0.7.2

_Make all requested tool calls and gather the output for a single prompt_

## 0.7.1

_Redo `grep` tool and update rustyline_

### Fixed

- Run grep against knowledge dir
- Don't panic if `grep` fails

### Added

- Tools documentation

## 0.7.0

_Initial support for internal tools for Ollama models_

### Added

- Check if current model has `tools` capability and enable tool calls if it does
- Report tool calls when they are made
- Add the following basic tools for Ollama models:
    - `grep`
    - `pwd`
    - `Git Status`
    - `Git Diff`
- `tools` command to list available tools

## 0.6.0

_Add profiles_

### Breaking changes

The previous model configuration system has been replaced with profiles.

### Fixed

Running `cforge` without a config file creates a new one with the default values.

### Profiles

- Add a profile system to allow storing multiple different configurations
- Profiles can define multiple models with up to three types
    - Fast
    - Balanced
    - Deep
    - Types must be distinct for each profile
    - Defaults to `balanced` if not specified
- Defaults to `local` profile if no profile is specified
    - model: `gemma3:12b`
    - model_type: `balanced`

### Commands

#### `:model <optional model_type>`

Switch the model type used by the current profile, if no model type is specified, print the profile's models.

Model types are:

- `fast`
- `balanced`
- `deep`

#### `:profile <optional profile_name>`

Switch the profile used by the current session, if no profile is specified, print the available profiles and their
models.

### Config

Example of two profiles with one and two models:

```toml
[profiles_config]

[[profiles_config.profiles]]
name = "local"
provider = "ollama"

[[profiles_config.profiles.models]]
model = "model1"
model_type = "fast"

[[profiles_config.profiles.models]]
model = "model2"
# model_type is optional, defaults to balanced

[[profiles_config.profiles]]
name = "remote"
provider = "anthropic"

[[profiles_config.profiles.models]]
model = "model1"
model_type = "fast"
```

## 0.5.0

_Add prompt command!_

### Breaking changes

- Move default chat data folder from `xdg/cforge` to `xdg/cforge/chats` to clearly separate `xdg/cforge/prompts`.

### `:prompt`

Reuse or edit prompts using a command. Use `${{user_prompt}}` to control the actual prompt placement.

```
# use an existing prompt. Prompt file can contain ${{user_prompt}} to position the prompt, 
otherwise it is appended after the prompt file
:prompt /path/to/file
User prompt to send along the selected prompt

# edit/create a prompt
:prompt relative/path
:prompt /absolute/path
```

## 0.4.3

_Append additional content to the current user prompt instead of giving it separately_

## 0.4.2

_Add configurable default completion prefixes_

### Config

Allow user to configure what (if any) path is automatically appended to a command when completion is used.

```toml
[command_prefixes]
switch = "@c/"
list = "@c/"
context = "@k/"
```

## 0.4.1

_Separate cache configuration_

### Config

- Removed `cforge_dir`, just use XDG data directory instead

## 0.4.0

_Add support for Anthropic API_

### Config

* `provider` - Change model provider between `ollama` and `anthropic`
* `max_tokens` - Set the token limit for Anthropic API

## 0.3.1

_Switch to XDG-compliant directory structure_

### Dirs

- **Chat**: `~/.local/share/cforge`
- **Config**: `~/.config/cforge/`

## 0.3.0

_Enable switching context file with a command and add support for a separate knowledge directory_

### Config

`knowledge_dir` - Add a path alias, which can be expanded using command completion

### Path aliases

Support path aliases instead of defaulting to cforge_dir.

* "/" - Absolute path
* "" - Relative to the current dir
* "@c/" - Relative to cforge_dir
* "@k/" - Relative to knowledge_dir

## 0.2.6 - 2025-06-26

_Refactoring and context file improvement_

### Context file

- Reload the file on each loop
- Include the context file in the token estimation
- Pass the context file as a user prompt instead

## 0.2.5 - 2025-06-21

_Remember the previous history file_

### Args

Make HISTORY_FILE arg optional if the user has opened another file previously.

## 0.2.4 - 2025-06-21

_Display estimated token count_

Optional print to show an estimation of history size as tokens (1 token ≈ 4 characters).

## 0.2.3 - 2025-06-21

_Refactor commands to use structs_

## 0.2.2 - 2025-06-19

_Support newlines using ALT + ENTER_

## 0.2.1 - 2025-06-15

_Implement directory completion for commands_

### Commands

Directory completion works for absolute and relative paths (in relation to `cforge_dir`) for `:list` and `:switch`
commands.

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
