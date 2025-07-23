# TODO

- [x] Clarify how the prompt is formed
- [x] Add a configuration file
- [x] Integrate rustyline
- [x] Use ollama API instead of run commands
- [x] Parse the chat history to a correctly formatted JSON
- [x] Implement simple completions with rustyline
    - [x] Commands
    - [x] Files
- [x] Support multiline input with alt + enter (using rustyline)
- [x] Update the default sysprompt
- [x] Keep track of the model's context window and file size
- [ ] Add `keep_alive` configuration that is sent with the API requests
- [ ] Add support for knowledge directory
- [ ] Re-implement AI response interruption
- [ ] Add functionality to truncate a chat
- [ ] Create memories, which are included in the prompt by default (session/global)

## Commands

- [ ] Allow changing the context file during a chat
    - [x] `config.create_editor` - Handle command/file command logic using the registry
    - Only support absolute paths initially
    - When support for knowledge directory is added, support relative paths
- [ ] `:copy` Copy the history file to another location. Edit the copy of the file?
- [ ] Absolute file path completion does not work
- [ ] Use prompt files with the current message
- [ ] Truncate chat (line count, estimated tokens, or LLM assisted)

## Completion overhaul

Instead of having just absolute and cforge dirs, there should be few options for static dirs.

Certain commands can then be completed by defaulting to a corresponding path.
* e.g. `switch` to cforge_dir and `context` to knowledge_dir

For simplicity, just expand the aliases to absolute paths on completion request

### Dirs
* "/" - Absolute path
* "" - Relative to the current dir
* "@c/" - Relative to cforge_dir
* "@k/" - Relative to knowledge_dir

### Config
Maybe just adding default prefixes to the toml

```toml
switch = "@c/"
context = "@k/"
```

