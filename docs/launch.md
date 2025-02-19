# Example VSCode Launch Config

Please replace the OPEN_API_KEY with your key.

    {
        // Use IntelliSense to learn about possible attributes.
        // Hover to view descriptions of existing attributes.
        // For more information, visit: https://go.microsoft.com/fwlink/?linkid=830387
        "version": "0.2.0",
        "configurations": [
            {
                "type": "lldb",
                "request": "launch",
                "name": "Debug executable 'iamcommitted'",
                "cargo": {
                    "args": [
                        "build",
                        "--bin=iamcommitted",
                        "--package=iamcommitted"
                    ],
                    "filter": {
                        "name": "iamcommitted",
                        "kind": "bin"
                    }
                },
                "args": [],
                "cwd": "${workspaceFolder}",
                "env": {
                    "OPENAI_API_KEY": "<<REPLACE-ME>>",
                    "RUST_BACKTRACE": "1", // Enable backtraces
                    "MY_CUSTOM_VAR": "some_value",  // Your custom environment variables here
                    "CARGO_MANIFEST_DIR": "${workspaceFolder}", // Very helpful for projects using relative paths in Cargo.toml
                },
            },
            {
                "type": "lldb",
                "request": "launch",
                "name": "Debug unit tests in executable 'iamcommitted'",
                "cargo": {
                    "args": [
                        "test",
                        "--no-run",
                        "--bin=iamcommitted",
                        "--package=iamcommitted"
                    ],
                    "filter": {
                        "name": "iamcommitted",
                        "kind": "bin"
                    }
                },
                "args": [],
                "cwd": "${workspaceFolder}"
            }
        ]
    }