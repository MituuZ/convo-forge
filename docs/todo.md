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
- [x] Add support for knowledge directory
- [ ] The model might not realize that it has the context file available (improve prompt; either system or context file)
- [ ] Does Rustyline support case insensitive completion?
- [ ] Keybinds for commands?
- [ ] Maybe later support streaming the responses
- [ ] Support memories, which are included in the prompt by default (session/global)
- [ ] Add Anthropic context sizes manually? There doesn't seem to be an API

## Commands

- [x] Allow changing the context file during a chat
    - [x] `config.create_editor` - Handle command/file command logic using the registry
- [ ] `:copy` Copy the history file to another location. Edit the copy of the file?
- [ ] Use prompt files with the current message
- [ ] Truncate chat (line count, estimated tokens, or LLM assisted)

## Completion overhaul
- [ ] Default prefix handling (when user completes a command, also insert the prefix)
- [ ] Allow configuring the default prefix for each command

### Config
Default dir would be just a tab away

```toml
# :switch @c/
switch = "@c/"
# :context @c/
context = "@k/"
```

