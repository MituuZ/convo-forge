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
- [ ] Update the default sysprompt
- [ ] Add `keep_alive` configuration that is sent with the API requests
- [ ] Add support for knowledge directory
- [ ] Re-implement AI response interruption
- [ ] Add functionality to truncate a chat
- [ ] Keep track of the model's context window and file size
- [ ] Create memories, which are included in the prompt by default (session/global)

## Commands

- [ ] Copy the history file to another location
- [ ] Allow changing the context file during a chat
- [ ] Use prompt files with the current message
- [ ] Truncate chat (line count, estimated tokens, or LLM assisted)
