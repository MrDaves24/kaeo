# Keep an eye on
Keep an eye on a folder, folders, a file or files and run a command when anything changes.

# Help
```shell
Usage: kaeo [OPTIONS] <COMMAND> <PATH>...

Arguments:
  <COMMAND>  Command to run when a file or folder changes
             Use {} to include the path that changed
             Use {@} to include all watched paths, as separate arguments
             Don't quote placeholders, paths are quoted when needed
  <PATH>...  Paths to watch for changes

Options:
  -r, --recursive                  Give the file that changed to {} and not the watched path containing it
  -I, --placeholder <PLACEHOLDER>  Placeholder for the changed path, if the command needs a literal {} [default: {}]
  -h, --help                       Print help
  -V, --version                    Print version
```

# Usage
What command runs when `src/main.rs` changes ?

- No path : `kaeo "du -hs" src/ Cargo.toml`
  - `du -hs`
- Watched path : `kaeo "du -hs {}" src/ Cargo.toml`
  - `du -hs src/`
- Changed file : `kaeo -r "du -hs {}" src/ Cargo.toml`
  - `du -hs src/main.rs`
- All paths : `kaeo "du -hs {@}" src/ Cargo.toml`
  - `du -hs src/ Cargo.toml`
- Placeholder inside an argument : `kaeo -r "diff {} {}.orig" src/`
  - `diff src/main.rs src/main.rs.orig`
- Command needing a literal `{}` : `kaeo -I @ "jq '{}' @" data.json`
  - `jq '{}' data.json` when `data.json` changes

# Notes
Don't put quotes around placeholders: paths are added as unique arguments, even if they contain spaces.
Paths are given as you typed them, relative to where kaeo was started.
The command also runs once at startup, or once per watched path when it uses `{}`.
