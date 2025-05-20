#!/bin/sh
#
# Git hook to prepare commit message using i-am-committed
#
# This hook is called by 'git commit' with up to three arguments:
# 1. The name of the file that contains the commit log message.
# 2. The type of commit (e.g., message, template, merge, squash, commit).
# 3. The commit SHA-1 if -c, -C, or --fixup is given.

# Assuming the iamcommitted binary is in the target/release directory
# and the hook is run from the root of the repository.
# Adjust the path to the binary if necessary.
EXECUTABLE_PATH="./target/release/iamcommitted"

# Check if the executable exists
if [ ! -x "$EXECUTABLE_PATH" ]; then
  # Fallback to debug if release is not found
  EXECUTABLE_PATH="./target/debug/iamcommitted"
  if [ ! -x "$EXECUTABLE_PATH" ]; then
    echo "Error: iamcommitted executable not found or not executable at $EXECUTABLE_PATH or ./target/release/iamcommitted."
    echo "Please build the project using 'cargo build' or 'cargo build --release'."
    exit 1
  fi
fi

# Pass all arguments to the iamcommitted binary
# The binary should be designed to handle these arguments
# and write the generated message to the file specified by $1.
"$EXECUTABLE_PATH" prepare-commit-msg "$@"

# Exit with the status of the iamcommitted binary
exit $?