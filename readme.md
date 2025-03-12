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
    - [Build and Install](#build-and-install)
    - [Running the Application from command line](#running-the-application-from-command-line)
  - [Unit Tests](#unit-tests)
    - [References](#references)

## Setup

To set up the repository, clone it to your local machine:

```sh
git clone https://github.com/<github>/iamcommitted.git
cd iamcommitted
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