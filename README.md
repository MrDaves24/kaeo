# Keep an eye on
Keep an eye on a folder, folders, a file or files and run a command when anything changes.

# Help
```shell
Usage: kaeo [OPTIONS] <COMMAND> <PATH>...

Arguments:
  <COMMAND>     Command to run when a file or folder changes
                Use {} to include the path in the command
                Use % to include all specified paths as separate arguments
                Use %% to include all specified paths as one argument
  <PATH>...     Paths to watch for changes

Options:
  -r, --recursive  Run command on the file that changed and not the parent (works with {} placeholder)
  -h, --help       Print help
  -V, --version    Print version
```

# Usage
What command runs when `src/main.rs` changes ?

- No path : `kaeo "du -hs" src/ Cargo.toml`
  - `du -hs`
- One path : `kaeo "du -hs {}" src/ Cargo.toml`
  - `du -hs src/`
- One path, recursive : `kaeo -r "du -hs {}" src/ Cargo.toml`
  - `du -hs src/main.rs`
- All paths : `kaeo "du -hs %" src/ Cargo.toml`
  - `du -hs src/ Cargo.toml`
- All paths as one argument : `kaeo "du -hs %%" src/ Cargo.toml`
  - `du -hs "src/ Cargo.toml"`

# Notes
No need to add quotes around the path (`{}`), the path is added as a unique argument, even if it contains spaces.
