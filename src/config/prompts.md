# Commit Message Prompts

## System Prompt

Generate a git commit message following the Conventional Commits specification. Use one of these types: feat, fix, chore, docs, style, refactor, perf, test, build, ci, revert. Include a scope in parentheses if relevant. Example format: type(scope): description
DO NOT provide any advice in the commit message. Only account for the changes made.

## User Prompt

Please analyze the following git diff and generate a commit message that follows the conventional commit format. The message should be clear, concise, and meaningful, helping developers understand the changes made:

{diff}
