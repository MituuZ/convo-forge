# Changelog

## 0.1.4 - 2025-06-08
_Add `switch` command_

### Commands
- Enables changing the history file during an active chat

## 0.1.3 - 2025-06-07
_Add `list` command_

### Commands
- First active chat command `:list`, list files from your sllama_dir with an optional parameter to filter
- Modify exit to a command `:q`

## 0.1.2 - 2025-06-06
_Implement history file_

### Configs
- `sllama_dir` - Now works as expected. Relative files are created here, but absolute paths are respected

## 0.1.1 - 2025-06-06
_Add configuration and changelog files_

### Configs
- `sllama_dir` - Not applied yet, will be default location for history files
- `model` - Switch used model
- `system_prompt` - Customize the prompt that sllama provides to the model

## 0.1.0 - 2025-06-05
_Initial release_

### Features
- Actual files as chat history
- Add context from a file to each chat message
