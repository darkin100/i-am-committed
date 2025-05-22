#!/bin/sh
#
# Git hook to prepare commit message using i-am-committed
#
# This hook is called by 'git commit' with up to three arguments:
# 1. The name of the file that contains the commit log message.
# 2. The type of commit (e.g., message, template, merge, squash, commit).
# 3. The commit SHA-1 if -c, -C, or --fixup is given.

# Assuming the iamcommitted binary is installed in /usr/local/bin
# Adjust the path to the binary if necessary.
EXECUTABLE_PATH="/usr/local/bin/iamcommitted"

# Check if the executable exists and is executable
if [ ! -x "$EXECUTABLE_PATH" ]; then
  echo "Error: iamcommitted executable not found or not executable at $EXECUTABLE_PATH."
  echo "Please ensure 'iamcommitted' is built and installed to /usr/local/bin."
  echo "You can build with 'cargo build --release' and then copy 'target/release/iamcommitted' to '/usr/local/bin/',"
  echo "or use 'cargo install --path .' if your cargo bin directory is in your PATH and then symlink/copy from there."
  exit 1
fi

# Pass all arguments to the iamcommitted binary
# The binary should be designed to handle these arguments
# and write the generated message to the file specified by $1.
"$EXECUTABLE_PATH" prepare-commit-msg "$@"

# Exit with the status of the iamcommitted binary
exit $?