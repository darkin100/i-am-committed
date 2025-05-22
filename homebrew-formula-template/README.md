# Homebrew Tap for iamcommitted

This directory contains a template for setting up a Homebrew tap repository for the `iamcommitted` application.

## Setting Up Your Homebrew Tap

1. Create a new GitHub repository named `homebrew-iamcommitted` (the `homebrew-` prefix is required by Homebrew)

2. Initialize the repository with the following structure:
   ```
   homebrew-iamcommitted/
   ├── Formula/
   │   └── iamcommitted.rb
   └── README.md
   ```

3. Copy the `iamcommitted.rb` formula template from this directory to your repository's `Formula/` directory

4. Update the formula with your specific details:
   - Update the `homepage` to point to your GitHub repository
   - For the first release, you'll need to manually update the `sha256` after creating your first release
   - Update the `license` field to match your project's license

5. Create a GitHub Personal Access Token with `repo` scope:
   - Go to GitHub Settings > Developer settings > Personal access tokens
   - Generate a new token with `repo` scope
   - Copy the token

6. Add the token as a secret in your main repository:
   - Go to your main repository settings
   - Navigate to Secrets and variables > Actions
   - Create a new repository secret named `HOMEBREW_TAP_TOKEN`
   - Paste your Personal Access Token as the value

## How the GitHub Actions Workflow Works

The GitHub Actions workflow you've set up in your main repository will:

1. Build your application for both Intel and Apple Silicon Macs
2. Create a universal binary that works on both architectures
3. Package the binary in a tarball and generate checksums
4. Create a GitHub release with the binary and checksums
5. Update your Homebrew formula with the new version, URL, and SHA256

## Triggering a Release

To trigger a release and update your Homebrew formula:

1. Tag your repository with a version number:
   ```bash
   git tag v1.0.0
   git push origin v1.0.0
   ```

2. The GitHub Actions workflow will automatically:
   - Build the application
   - Create a release
   - Update your Homebrew formula

## Installing Your Application via Homebrew

Once your tap is set up and your first release is published, users can install your application with:

```bash
brew tap darkin100/iamcommitted
brew install iamcommitted
```

## Troubleshooting

- **First Release**: For the first release, you may need to manually create the formula in your tap repository. After that, the GitHub Actions workflow will update it automatically.
- **SHA256 Mismatch**: If you see SHA256 mismatch errors, ensure the workflow correctly calculated and updated the SHA256 in your formula.
- **Permission Issues**: If the workflow fails to push to your tap repository, check that the `HOMEBREW_TAP_TOKEN` has the correct permissions.
