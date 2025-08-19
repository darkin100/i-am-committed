# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build and Development Commands

### Building
```bash
cargo build          # Debug build
cargo build --release  # Release build
```

### Testing
```bash
cargo test          # Run all tests
cargo test -- --nocapture  # Run tests with output
```

### Linting and Formatting
```bash
cargo fmt           # Format code
cargo clippy        # Run linter
```

### Running the Application
```bash
# Development mode
cargo run -- [args]

# Release mode
cargo run --release -- [args]

# With verbose logging
cargo run -- -v
```

## Architecture Overview

iamcommitted is an AI-powered Git commit message generator built in Rust. The codebase follows a modular architecture with clear separation of concerns:

### Core Modules

1. **`src/main.rs`** - CLI entry point
   - Handles command-line arguments via clap
   - Manages interactive and Git hook modes
   - Sets up logging infrastructure

2. **`src/git/mod.rs`** - Git operations
   - `GitClient` provides abstraction over Git commands
   - Handles staged changes, commits, and repository state
   - Contains comprehensive test suite using temporary repositories

3. **`src/ai/mod.rs`** - AI integration
   - `AIClient` manages OpenAI API interactions
   - Supports custom endpoints and model selection
   - Loads prompts from configuration files

4. **`src/commit_formatter/`** - Message formatting
   - Cleans and formats AI-generated commit messages
   - Removes artifacts like backticks and excess whitespace

### Configuration

- **Environment Variables**:
  - **API Key** (required, checked in order):
    - `IAC_OPENAI_API_KEY` (takes precedence if set)
    - `OPENAI_API_KEY` (fallback)
  - **Model** (optional, defaults to `gpt-4o-mini`):
    - `IAC_OPENAI_MODEL` (takes precedence if set)
    - `OPENAI_MODEL` (fallback)
  - **Endpoint** (optional, for custom endpoints):
    - `IAC_OPENAI_ENDPOINT` (takes precedence if set)
    - `OPENAI_ENDPOINT` (fallback)

The `IAC_OPENAI_*` prefixed variables allow users to configure IAmCommitted-specific OpenAI settings without affecting other applications that may use the standard `OPENAI_*` variables.

- **Prompts**: Located in `src/config/prompts.md`, uses markdown format with system and user sections

### Testing Strategy

Tests are co-located with modules using Rust's `#[cfg(test)]` pattern. Key test areas:
- Git operations use temporary repositories for isolation
- AI client tests verify configuration and initialization
- Formatter tests ensure proper message cleaning

Run a specific test:
```bash
cargo test test_name
```

### Git Hook Integration

The application can run as a `prepare-commit-msg` hook:
- Hook script: `hooks/prepare-commit-msg.sh`
- Expects binary at `/usr/local/bin/iamcommitted`
- Automatically generates messages when no `-m` flag is used

### Logging

Logs are written to `~/.iamcommitted/logs/` with timestamp-based filenames. Use `-v` flag for verbose output to console.

## Git Commit Workflow

When creating Git commits in this repository, use the `iamcommitted` tool instead of the standard `git commit` command:

```bash
# Stage your changes first
git add [files]

# Then use iamcommitted to generate and create the commit
iamcommitted

# Or in verbose mode
iamcommitted -v
```

The `iamcommitted` tool will:
1. Analyze the staged changes
2. Generate an appropriate commit message using AI
3. Create the commit with the generated message

This ensures consistent, descriptive commit messages that follow best practices.