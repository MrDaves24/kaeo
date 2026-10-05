# Keep an eye on
Keep an eye on a folder, folders, a file or files and run a command when anything changes.

# Help
```shell
Usage: kaeo [OPTIONS] <COMMAND> <PATH>...

Arguments:
  <COMMAND>  Command to run when a file or folder changes, with sh -c (pipes, &&, redirects work)
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
- Shell syntax : `kaeo -r "wc -l {} && cargo test 2>&1 | tail -1" src/`
  - `sh -c "wc -l src/main.rs && cargo test 2>&1 | tail -1"`
- Command needing a literal `{}` : `kaeo -I @ "jq '{}' @" data.json`
  - `jq '{}' data.json` when `data.json` changes

# Notes
The command runs with `sh -c`, so pipes, `&&`, redirects, `$VAR` and globs work. Need zsh or bash syntax? Put it in a script with a shebang: `kaeo -r "./check.zsh {}" src/`, the path is `$1` in the script.
Don't put quotes around placeholders: paths are added as unique arguments, even if they contain spaces.
Paths are given as you typed them, relative to where kaeo was started.
The command also runs once at startup, or once per watched path when it uses `{}`. With `-r`, watched folders are skipped at startup, as `{}` is meant to be a file.

# Changelog
## 2.0.0
Breaking changes:
- `%` → `{@}`; `%%` removed
- Commands run through `sh -c`: pipes, `&&`, redirects work; `$`, `*`, `;` are now interpreted by the shell
- `{}` gives the path as typed (`src/main.rs`), not the absolute path

Other changes:
- `{}` works anywhere in an argument and any number of times; `-I` changes it
- With `-r`, startup skips watched folders
- One run per batch of changes; vim and atomic saves are detected
- Exits with code 1 on startup errors
