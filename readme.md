# I am Committed

## An AI micro bot for generating Git Commits

This project is an experimentation into AI. Specifically I am looking to understand AI tools better to answer 2 questions.

1. Are AI Coding tools any good and do they make me more productive?
2. How does application development change when you build AI (LLM) enhanced products.

I-a-committed is a simple tool to solve a simple problem that I have seen over and over again.  Poor Git commit!!  Which cause further downstream problems, everything from poor merge requests to release notes. All adding to reduced productivity.  Can AI & LLM help?

Another interesting aspect of this project is that I am using it to generate my own commit messages for this project, so the Git commit history will show the evolution of the results.

Areas of learning area:

- Rust
- LLM different performance characteristics
- LLM architectures, cloud vs local
- Prompt Engineering
- Fine Tuning (maybe)

## Setup

To set up the repository, clone it to your local machine:

    git clone https://github.com/<github>/iamcommitted.git
    cd iamcommitted

### Build and Install

To build and install the command line tool, use the following command:

    cargo install --path .

### Running the Application from command line

The CLI currently uses OpenAIs API. So you will need to get your [own API key](https://platform.openai.com/) and set it as an environment variable, or as part of your vscode launch.json

    export OPENAI_API_KEY=<key>

To execute the command, its as simple as:

    git add .
    iamcommitted

This should generate a commit message following the conventional commits standard.

    refactor(commit_formatter): remove unused import of CommitType

    This commit removes the unused import of `CommitType` from the `commit_formatter` module, helping to clean up the code and improve readability.

    Please select an option:
    1. Commit changes with this message
    2. Cancel

    Enter your choice (1-2):

## Unit Tests

We have used Cline to generate unit tests, you can test them running the cargo command.

    cargo test

### References

Conventional commits

<https://www.conventionalcommits.org/en/v1.0.0/#>

Inspiration comes from the great work that done by

<https://github.com/Nutlope/aicommits>


Testing Testing