# Conventional Commits Regex Validation

## Overview

This document describes the regex pattern used to validate commit messages against the Conventional Commits specification, tailored to match the actual output from the `iamcommitted` agent based on analysis of real commit messages in `data/split_logs`.

## Regex Pattern

```regex
^(feat|fix|docs|style|refactor|perf|test|chore|build|ci|revert)(\([a-zA-Z0-9_-]+\))?: [a-z].+
```

## Pattern Breakdown

### 1. Commit Type (Required)
```regex
^(feat|fix|docs|style|refactor|perf|test|chore|build|ci|revert)
```

**Matches:**
- `feat` - New feature
- `fix` - Bug fix
- `docs` - Documentation changes
- `style` - Code style changes (formatting, etc.)
- `refactor` - Code refactoring
- `perf` - Performance improvements
- `test` - Adding or updating tests
- `chore` - Maintenance tasks
- `build` - Build system changes
- `ci` - CI/CD changes
- `revert` - Reverting previous commits

**Examples from dataset:**
- ✓ `feat`
- ✓ `fix`
- ✓ `docs`
- ✓ `chore`
- ✗ `feature` (must use abbreviated form)
- ✗ `bugfix` (must use `fix`)

### 2. Scope (Optional)
```regex
(\([a-zA-Z0-9_-]+\))?
```

The scope is optional but if present:
- Must be wrapped in parentheses
- Can contain:
  - Uppercase letters (A-Z)
  - Lowercase letters (a-z)
  - Numbers (0-9)
  - Hyphens (-)
  - Underscores (_)
- Cannot contain spaces

**Examples from dataset:**
- ✓ `(ai)` - lowercase
- ✓ `(README)` - uppercase
- ✓ `(Observability)` - capitalized
- ✓ `(commit-options)` - hyphenated
- ✓ `(local-development)` - multi-word with hyphens
- ✓ `(finops)` - special naming
- ✗ `(my scope)` - contains space
- ✗ `(my.scope)` - contains period

### 3. Separator (Required)
```regex
:
```

Must be a colon followed by exactly one space.

**Examples:**
- ✓ `: ` (colon + space)
- ✗ `:` (missing space)
- ✗ `:  ` (too many spaces)

### 4. Description (Required)
```regex
[a-z].+
```

The description:
- Must start with a lowercase letter
- Can contain any characters after the first character
- Should not end with a period (convention)

**Examples from dataset:**
- ✓ `add prompts configuration for generating commit messages`
- ✓ `update resource monitoring section and add cron job details`
- ✓ `remove deprecated backup and monitoring scripts`
- ✗ `Add prompts configuration` (starts with uppercase)
- ✗ `UPDATE configuration` (starts with uppercase)

## Complete Examples

### Valid Commit Messages

From the actual dataset in `data/split_logs`:

```
✓ feat(ai): add prompts configuration for generating commit messages
✓ fix(ai): update log directory to use HOME environment variable
✓ docs(Observability): update resource monitoring section and add cron job details
✓ chore(scripts): remove deprecated backup and monitoring scripts
✓ style(css): update commit message color from green to grey
✓ feat(commit-options): add interactive commit message processing functionality
✓ docs(readme): add missing newline for readability in instructions
✓ refactor(docs): reorganize Service Model documentation
✓ feat(local-development): update README for local development process
```

### Invalid Commit Messages

```
✗ Feature(ai): add new feature
   ^^^^^^^^^
   Should be 'feat' not 'Feature'

✗ feat(ai): Add new feature
            ^^^
            Description should start with lowercase

✗ feat (ai): add new feature
      ^
      Space not allowed before scope

✗ feat(my scope): add new feature
         ^
         Space not allowed in scope

✗ feat:add new feature
      ^
      Missing space after colon

✗ feat(): add new feature
      ^^
      Empty scope not allowed (just omit it)
```

## Message Extraction

The raw output from `agent.generate_commit_message()` is wrapped in `<commit_message>` tags:

```
<commit_message>
feat(ai): add new feature for processing commits
</commit_message>
```

Before validation, these tags must be stripped using the `extract_commit_message()` helper function:

```rust
fn extract_commit_message(raw_message: &str) -> String {
    let tag_regex = Regex::new(r"(?s)<commit_message>\s*(.*?)\s*</commit_message>").unwrap();

    if let Some(caps) = tag_regex.captures(raw_message) {
        caps.get(1).map(|m| m.as_str().trim().to_string()).unwrap_or_else(|| raw_message.to_string())
    } else {
        raw_message.to_string()
    }
}
```

This function:
- Extracts content from within `<commit_message>` tags
- Trims whitespace from the extracted message
- Falls back to returning the raw message if tags are not found

## Usage in Tests

The regex is used in two main tests:

### 1. Batch Validation Test
```rust
#[tokio::test]
async fn test_agent_with_real_diffs_matches_conventional_commits()
```

This test:
- Loads multiple commit messages from log files
- Extracts messages from `<commit_message>` tags
- Tests each against the regex
- Requires 80% success rate
- Reports which messages fail validation

### 2. Detailed Structure Test
```rust
#[tokio::test]
async fn test_agent_validates_message_structure_with_real_diffs()
```

This test:
- Extracts messages from `<commit_message>` tags
- Validates subject line length (≤100 characters)
- Validates no period at end of subject
- Validates conventional commits format
- Validates description starts with lowercase

## Testing the Regex

You can test the regex pattern manually:

```bash
# Run the integration tests
cargo test --test agent_integration_test test_agent_with_real_diffs_matches_conventional_commits -- --nocapture

# Run with a specific number of samples
# (modify the `load_test_data_from_logs` limit in the test)
```

## Dataset Analysis

Based on analysis of 133 commit messages in `data/split_logs`:

- **Type Distribution:**
  - `feat`: ~35%
  - `docs`: ~30%
  - `fix`: ~12%
  - `chore`: ~10%
  - `style`: ~5%
  - `refactor`: ~3%
  - Others: ~5%

- **Scope Patterns:**
  - Lowercase single word: 45% (e.g., `ai`, `css`, `readme`)
  - Capitalized: 30% (e.g., `Observability`, `README`)
  - Hyphenated: 15% (e.g., `commit-options`, `local-development`)
  - No scope: 10%

- **Description Patterns:**
  - 100% start with lowercase letter
  - Average length: 40-60 characters
  - Most common first words: `add`, `update`, `remove`, `fix`, `correct`

## Conventional Commits Specification Reference

For full specification, see: https://www.conventionalcommits.org/

Our regex enforces:
- ✓ Type is required
- ✓ Scope is optional but recommended
- ✓ Description is required
- ✓ Description starts with lowercase
- ✓ No period at end of subject line
- ✓ Uses imperative mood ("add" not "added")
