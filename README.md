# PromptBank

A Rust CLI tool to manage and apply prompts for Claude AI. Store, organize, and quickly apply system prompts, skills, agent configurations, roles, and more.

## Features

- **Multiple prompt categories**: system, skill, agent, role, task, template, custom
- **Variable templating**: Use `{{variable}}` syntax in prompts
- **Search and filter**: Find prompts by name, description, tags, or content
- **Import/Export**: Share prompt collections as JSON files
- **Clipboard support**: Copy prompts directly to clipboard
- **Interactive mode**: Fill in variables interactively

## Installation

```bash
cargo build --release
# Binary will be at ./target/release/promptbank
```

## Usage

### Add a prompt

```bash
# Interactive mode
promptbank add

# With flags
promptbank add --name "my-prompt" --category "system" --description "Description" --content "Prompt content"

# From file
promptbank add --name "my-prompt" --category "role" --description "A role prompt" --file ./prompt.txt

# With tags
promptbank add --name "my-prompt" --category "skill" --description "Skill prompt" --tags "coding,review"
```

### List prompts

```bash
# List all
promptbank list

# Filter by category
promptbank list --category system

# Show full content
promptbank list --full
```

### Get a prompt

```bash
# By ID or name
promptbank get my-prompt

# Copy to clipboard
promptbank get my-prompt --copy

# Raw output (for piping)
promptbank get my-prompt --raw
```

### Apply a prompt (with variable substitution)

```bash
# Basic apply
promptbank apply my-template

# With variables
promptbank apply my-template --var "name=John" --var "task=Review"

# Interactive variable input
promptbank apply my-template --interactive

# Copy result to clipboard
promptbank apply my-template --var "name=John" --copy
```

### Edit a prompt

```bash
promptbank edit my-prompt
```

### Delete a prompt

```bash
promptbank delete my-prompt

# Skip confirmation
promptbank delete my-prompt --force
```

### Search prompts

```bash
promptbank search "code review"
```

### Export/Import

```bash
# Export all prompts
promptbank export ./my-prompts.json

# Import prompts (replace all)
promptbank import ./my-prompts.json

# Merge with existing
promptbank import ./my-prompts.json --merge
```

### Show info

```bash
promptbank info
```

## Prompt Categories

| Category | Description |
|----------|-------------|
| `system` | System prompts that define AI behavior |
| `skill` | Specific skills or capabilities |
| `agent` | Agent configurations and personas |
| `role` | Professional roles and expertise |
| `task` | Task-specific instructions |
| `template` | Reusable templates with variables |
| `custom:name` | Custom categories |

## Variable Templating

Use `{{variable_name}}` in your prompt content:

```
You are a {{role}} expert helping with {{task}}.
```

Apply with:
```bash
promptbank apply my-prompt --var "role=Python" --var "task=debugging"
```

Or use interactive mode:
```bash
promptbank apply my-prompt --interactive
```

## Data Storage

Prompts are stored at:
- macOS: `~/Library/Application Support/com.claude.promptbank/prompts.json`
- Linux: `~/.local/share/promptbank/prompts.json`
- Windows: `%APPDATA%\claude\promptbank\data\prompts.json`

## License

MIT
