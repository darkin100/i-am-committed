# Packaging iamcommitted for Homebrew

This guide explains how to package the `iamcommitted` application for distribution via Homebrew on macOS.

## Overview

The following components have been set up to enable Homebrew distribution:

1. **GitHub Actions Workflow** (`.github/workflows/rust.yml`):
   - Builds the application for both Intel and Apple Silicon Macs
   - Creates a universal binary
   - Packages the binary and generates checksums
   - Creates GitHub releases
   - Updates the Homebrew formula automatically

2. **Homebrew Formula Template** (`homebrew-formula-template/`):
   - Contains a template for the Homebrew formula
   - Includes instructions for setting up a Homebrew tap repository

## Prerequisites

Before you can distribute your application via Homebrew, you need:

1. A GitHub repository for your application (which you already have)
2. A separate GitHub repository for your Homebrew tap (you'll need to create this)
3. A GitHub Personal Access Token with `repo` scope

## Step-by-Step Guide

### 1. Create a Homebrew Tap Repository

1. Create a new GitHub repository named `homebrew-iamcommitted`
2. Follow the instructions in `homebrew-formula-template/README.md` to set it up

### 2. Set Up GitHub Secrets

1. Create a GitHub Personal Access Token with `repo` scope
2. Add it as a secret named `HOMEBREW_TAP_TOKEN` in your main repository

### 3. Update the GitHub Actions Workflow (if needed)

The workflow file (`.github/workflows/rust.yml`) has been updated to:
- Build for both Intel and Apple Silicon Macs
- Create a universal binary
- Generate releases when you tag the repository
- Update your Homebrew formula automatically

You may need to update the `HOMEBREW_TAP_REPO` environment variable in the workflow file if your GitHub username is not `darkin100`.

### 4. Create Your First Release

1. Tag your repository with a version number:
   ```bash
   git tag v1.0.0
   git push origin v1.0.0
   ```

2. The GitHub Actions workflow will automatically:
   - Build the application
   - Create a release
   - Update your Homebrew formula (for non-alpha releases)

### 5. Testing the Installation

After your first release, you can test the installation with:

```bash
brew tap darkin100/iamcommitted
brew install iamcommitted
```

## How Users Will Install Your Application

Once everything is set up, users can install your application with:

```bash
brew tap darkin100/iamcommitted
brew install iamcommitted
```

Or if you eventually get your formula into Homebrew Core:

```bash
brew install iamcommitted
```

## Troubleshooting

- **First Release**: For the first release, you may need to manually create the formula in your tap repository. After that, the GitHub Actions workflow will update it automatically.
- **SHA256 Mismatch**: If you see SHA256 mismatch errors, ensure the workflow correctly calculated and updated the SHA256 in your formula.
- **Permission Issues**: If the workflow fails to push to your tap repository, check that the `HOMEBREW_TAP_TOKEN` has the correct permissions.
- **Universal Binary Issues**: If the universal binary creation fails, ensure you have the correct Rust targets installed.

## Resources

- [Homebrew Formula Cookbook](https://docs.brew.sh/Formula-Cookbook)
- [Homebrew Tap Guide](https://docs.brew.sh/Taps)
- [GitHub Actions Documentation](https://docs.github.com/en/actions)
