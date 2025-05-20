# I Am Committed

## An AI micro bot for generating Git Commits

This project is an experimentation into AI. Specifically, I am looking to understand AI tools better to answer two questions:

1. Are AI coding tools any good, and do they make me more productive?
2. How does application development change when you build AI (LLM) enhanced products?

I-am-committed is a simple tool to solve a common problem: poor Git commit messages. Poor commit messages can cause downstream problems, from poor merge requests to unclear release notes, all reducing productivity. Can AI & LLM help?

Another interesting aspect of this project is that I am using it to generate my own commit messages for this project, so the Git commit history will show the evolution of the results.

### Areas of Learning:

- Rust
- LLM performance characteristics
- LLM architectures (cloud vs local)
- Prompt Engineering
- Fine Tuning (maybe)

## Table of Contents

- [I Am Committed](#i-am-committed)
  - [An AI micro bot for generating Git Commits](#an-ai-micro-bot-for-generating-git-commits)
    - [Areas of Learning:](#areas-of-learning)
  - [Table of Contents](#table-of-contents)
  - [Setup](#setup)
    - [Install Rust \& Cargo](#install-rust--cargo)
    - [Build and Install](#build-and-install)
    - [Running the Application from command line](#running-the-application-from-command-line)
    - [Using as a `prepare-commit-msg` Hook](#using-as-a-prepare-commit-msg-hook)
  - [Unit Tests](#unit-tests)
    - [References](#references)

## Setup

To set up the repository, clone it to your local machine:

```sh
git clone https://github.com/darkin100/iamcommitted.git
cd iamcommitted
```

### Install Rust & Cargo

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Build and Install

To build and install the command line tool, use the following command:

```sh
cargo install --path .
```

### Running the Application from command line

The CLI currently uses OpenAIs API. So you will need to get your [own API key](https://platform.openai.com/) and set it as an environment variable, or as part of your vscode launch.json

```sh
export OPENAI_API_KEY=<key>
```

or update your terminal profile .zshrc file with your API Key.

```sh
echo 'export OPENAI_API_KEY="your_api_key_here"' >> ~/.zshrc
```

To execute the command, its as simple as:

```sh
git add .
iamcommitted
```

This should generate a commit message following the conventional commits standard.

```sh
refactor(commit_formatter): remove unused import of CommitType

This commit removes the unused import of `CommitType` from the `commit_formatter` module, helping to clean up the code and improve readability.
```

### Using as a `prepare-commit-msg` Hook

`i-am-committed` can also be used as a Git `prepare-commit-msg` hook to automatically generate a commit message before your editor opens.

#### Installation

1.  **Build the binary:**
    Ensure you have built the `i-am-committed` executable. You can do this with:
    ```sh
    cargo build
    ```
    Or for a release version:
    ```sh
    cargo build --release
    ```
    The hook script (`hooks/prepare-commit-msg.sh`) expects the binary to be in `target/debug/i-am-committed` or `target/release/i-am-committed`.

2.  **Make the hook script executable:**
    ```sh
    chmod +x hooks/prepare-commit-msg.sh
    ```

3.  **Install the hook:**
    Copy or symlink the provided `hooks/prepare-commit-msg.sh` script to your local repository's `.git/hooks/` directory.

    To copy:
    ```sh
    cp hooks/prepare-commit-msg.sh .git/hooks/prepare-commit-msg
    ```

    To symlink (recommended, so updates to the script in the repository are automatically reflected):
    ```sh
    ln -s -f ../../hooks/prepare-commit-msg.sh .git/hooks/prepare-commit-msg
    ```
    *(Ensure you run this command from the root of your repository.)*

#### Usage

Once installed, the hook will automatically run when you execute `git commit`.

- If you run `git commit` without `-m` or a template, `i-am-committed` will generate a message and write it to the commit message file. Your editor will then open with this pre-filled message.
- If you use `git commit -m "Your message"` or have a commit template configured, `i-am-committed` will not overwrite your message or template.

You still need to have your `OPENAI_API_KEY` environment variable set for the hook to function correctly.

## Unit Tests

We have used Cline to generate unit tests, you can test them running the cargo command.

```sh
cargo test
```

### References

Conventional commits

<https://www.conventionalcommits.org/en/v1.0.0/#>

Inspiration comes from the great work that done by

<https://github.com/Nutlope/aicommits>