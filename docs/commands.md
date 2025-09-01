# Commands

Commands can be entered during a chat by prepending the command with `:`. Commands are sensitive and support completion.

- help
- list
- switch
- edit
- exit
- sysprompt
- context
- prompt
- model
- profile

## Path aliases

These can be used to quickly find files from cforge and knowledge directories without having to write the full path.

- `/` - Absolute path
- ` ` - Relative to the current dir
- `@c/` - Expands to the data directory
- `@k/` - Expands to the knowledge directory
- `@p/` - Expands to the prompt directory

You can configure each file command with a custom prefix, either a path alias or absolute path. See readme Configuration
section.

Examples:

- `:swi <tab> :switch @c/`
- `:swi <tab> :switch /home/user/my_dir`

## Help

List available commands.

`:help`

## List

List all files in the data directory, optionally add a filter string.

`:list <filter>`

## Switch

Switch to a different history file. Supports either absolute or relative paths in the data directory.

`:switch relative/path`
`:switch /absolute/path`

Supports path aliases.

## Edit

Open the current history file in the user's editor.

Resolution order:

1. `$EDITOR`
2. `$VISUAL`
3. windows - `notepad` (untested)
4. other - `vi`

`:edit`

## Exit

Exit the current chat.

`:q`

## Sysprompt

Update the system prompt for this session. Does not modify any configurations.

`:sysprompt Enter the new system prompt here`

## Context

Change the context file for this session.

`:context relative/path`
`:context /absolute/path`

Supports path aliases.

## Prompt

Use or edit a prompt file. You can use `${{user_prompt}}` in a prompt file to control where the user prompt is inserted
when the message is sent. If it is not included, the user prompt is appended after the prompt file content.

To use a prompt file, write your actual prompt after the command and file. (e.g., using ALT + ENTER to move to the next
line)

```
:prompt /path/to/file
User prompt to send along the selected prompt
```

Edit a prompt by just calling

`:prompt relative/path`
`:prompt /absolute/path`

## Model

Switch or inspect the model type for the current profile. If no model type is specified, prints the profile's models.

Usage:

`:model <model_type>`

Model types are:

- `fast`
- `balanced`
- `deep`

Examples:

- `:model fast` — switch to the fast model configured for the active profile
- `:model` — print the models configured for the active profile

## Profile

Switch or inspect the active profile. If no profile is specified, prints the available profiles and their models.

Usage:

`:profile <profile_name>`

Examples:

- `:profile local`
- `:profile` — list profiles and their models
