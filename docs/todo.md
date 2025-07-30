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
- [x] The model might not realize that it has the context file available
- [ ] Keybinds for commands?
- [ ] Support memories, which are included in the prompt by default (session/global)
- [ ] Add Anthropic context sizes manually? There doesn't seem to be an API

## Commands

- [x] Allow changing the context file during a chat
    - [x] `config.create_editor` - Handle command/file command logic using the registry
- [ ] `copy` - Copy the history file to another location. Edit the copy of the file?
- [ ] `prompt`- Enable creating, editing and using prompt files
    - The user should be able to define where their actual prompt is injected
- [ ] Truncate chat (line count, estimated tokens, or LLM assisted)

## Completion overhaul

- [x] Default prefix handling (when user completes a command, also insert the prefix)
- [x] Allow configuring the default prefix for each command
