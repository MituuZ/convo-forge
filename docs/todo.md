# TODO

- [x] Clarify how the prompt is formed
- [x] Add a configuration file
- [x] Integrate rustyline
- [x] Use ollama API instead of run commands
- [ ] Parse the chat history to a correctly formatted JSON
- [ ] Delimiter customization
- [ ] Add `keep_alive` configuration that is sent with the API requests
- [ ] Implement completions with rustyline
    - [x] Commands
    - [ ] Files
- [ ] Support multiline input with shift + enter (using rustyline)
- [ ] Add support for knowledge directory
- [ ] Re-implement AI response interruption
- [ ] Add functionality to truncate a chat
- [ ] Keep track of the model's context window and file size
- [ ] Create memories, which are included in the prompt by default (session/global)

## Commands

- [ ] Copy the history file to another location
- [ ] Allow changing the context file during a chat
- [ ] Use prompt files with the current message
