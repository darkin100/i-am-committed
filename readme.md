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

To build the command line tool:

```sh
cargo build --release
```

This will create the executable at `target/release/iamcommitted`.

To make it accessible system-wide (and for the Git hook), you can copy it to a directory in your PATH, such as `/usr/local/bin`:

```sh
sudo cp target/release/iamcommitted /usr/local/bin/
```

Alternatively, `cargo install --path .` will install it to your Cargo binary directory (e.g., `~/.cargo/bin/`). If you use this method, ensure `~/.cargo/bin/` is in your system's PATH and the hook script is adjusted accordingly if you don't symlink/copy to `/usr/local/bin`. For simplicity, the provided hook script assumes `/usr/local/bin/iamcommitted`.

### Running the Application from command line

If you've installed `iamcommitted` to `/usr/local/bin` or another directory in your PATH, you can run it directly.

The CLI currently uses OpenAI's API. So you will need to get your [own API key](https://platform.openai.com/) and set it as an environment variable, or as part of your vscode launch.json

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

1.  **Build and Install the `iamcommitted` binary:**
    Ensure you have built the `iamcommitted` executable and placed it in `/usr/local/bin/`.
    ```sh
    cargo build --release
    sudo cp target/release/iamcommitted /usr/local/bin/
    ```
    Make sure `/usr/local/bin/iamcommitted` is executable. The `cp` command should preserve permissions, but you can verify with `ls -l /usr/local/bin/iamcommitted`.
    The hook script (`hooks/prepare-commit-msg.sh`) now expects the binary to be at `/usr/local/bin/iamcommitted`.

2.  **Make the hook script executable:**
    ```sh
    chmod +x hooks/prepare-commit-msg.sh
    ```

3.  **Install the Git hook script:**
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
