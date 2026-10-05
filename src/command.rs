use crossterm::{
    cursor::MoveTo,
    execute,
    terminal::{self, Clear, ClearType},
};
use gethostname::gethostname;
use std::{
    path::{Path, PathBuf},
    process::Stdio,
};

pub const ALL_PATHS: &str = "{@}";

pub struct Command {
    template: String,
    placeholder: String,
    all_paths: String,
    uses_placeholder: bool,
}

fn quote(path: &Path) -> String {
    shlex::try_quote(&path.to_string_lossy())
        .expect("paths can't contain NUL")
        .into_owned()
}

impl Command {
    pub fn new(template: String, placeholder: String, paths: &[PathBuf]) -> Option<Self> {
        let all_paths = paths.iter().map(|p| quote(p)).collect::<Vec<_>>().join(" ");
        let mut command = Self {
            template,
            placeholder,
            all_paths,
            uses_placeholder: false,
        };
        let (line, uses_placeholder) = command.substitute(&quote(Path::new("")));
        match shlex::split(&line) {
            None => {
                eprintln!("Invalid command, cannot parse");
                return None;
            }
            Some(args) if args.is_empty() => {
                eprintln!("Empty command");
                return None;
            }
            Some(_) => (),
        }
        command.uses_placeholder = uses_placeholder;
        Some(command)
    }

    /// Replaces placeholders in one pass, so inserted paths are never substituted again
    fn substitute(&self, current_path: &str) -> (String, bool) {
        let mut line = String::new();
        let mut used = false;
        let mut rest = self.template.as_str();
        while let Some(c) = rest.chars().next() {
            if let Some(r) = rest.strip_prefix(ALL_PATHS) {
                line.push_str(&self.all_paths);
                rest = r;
            } else if let Some(r) = rest.strip_prefix(self.placeholder.as_str()) {
                line.push_str(current_path);
                used = true;
                rest = r;
            } else {
                line.push(c);
                rest = &rest[c.len_utf8()..];
            }
        }
        (line, used)
    }

    fn args(&self, current_path: Option<&Path>) -> Option<Vec<String>> {
        let (line, _) = self.substitute(&current_path.map(quote).unwrap_or_default());
        shlex::split(&line).filter(|args| !args.is_empty())
    }

    pub fn run(&self, current_path: Option<&Path>, clean: bool) {
        // Prepare process
        let Some(args) = self.args(current_path) else {
            // Placeholder inside the user's own quotes, e.g. "{}" with a path containing "
            eprintln!("Cannot parse command after inserting paths, don't quote placeholders");
            return;
        };
        let mut process = std::process::Command::new(&args[0]);
        process
            .args(&args[1..])
            .stderr(Stdio::inherit())
            .stdout(Stdio::inherit());

        // Handle terminal
        if clean {
            execute!(std::io::stdout(), Clear(ClearType::All), MoveTo(0, 0)).ok(); // FUTURE : Error
            print_header(true, &process);
        } else {
            println!();
            print_header(false, &process);
        }

        // Spawn process
        let mut child = match process.spawn() {
            Ok(c) => c,
            Err(err) => {
                eprintln!("Failed to spawn process");
                eprint!("Error : {err:?}");
                return;
            }
        };
        let result = match child.wait() {
            Ok(c) => c,
            Err(err) => {
                eprintln!("Process crashed");
                eprintln!("Error : {err:?}");
                return;
            }
        };
        if !result.success() {
            println!("Error code : {result}");
        }
    }

    pub fn uses_placeholder(&self) -> bool {
        self.uses_placeholder
    }
}

fn print_header(with_right: bool, command: &std::process::Command) {
    let mut len_left = 0;
    print!("{}", command.get_program().display());
    len_left += command.get_program().len(); // Might be a source of error :) (len of storage != len displayed)

    for arg in command.get_args() {
        print!(" ");
        // TODO : Add quotes if it contains any whitespace
        print!("{}", arg.display());
        len_left += 1 + arg.len(); // Might again be a source of error
    }

    if !with_right {
        println!();
        return;
    }

    let x = terminal::size().unwrap_or((0, 0)).0 as usize;

    // Right header
    let hostname = gethostname();
    // len might be a source of error
    let right_len = hostname.len() + 19 + 2; // FUTURE : Might change with locale
    for _ in 0..(x.saturating_sub(right_len + len_left)) {
        print!(" ");
    }
    println!(
        "{}: {}",
        hostname.display(),
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(template: &str, placeholder: &str, paths: &[&str], current: &str) -> Vec<String> {
        let paths: Vec<PathBuf> = paths.iter().map(PathBuf::from).collect();
        let command = Command::new(template.into(), placeholder.into(), &paths).unwrap();
        command.args(Some(Path::new(current))).unwrap()
    }

    #[test]
    fn substitution() {
        let all = ["src/", "my dir"];
        assert_eq!(
            args("cp {} {}.bak", "{}", &all, "a.txt"),
            ["cp", "a.txt", "a.txt.bak"]
        );
        assert_eq!(
            args("fmt --file={}", "{}", &all, "a.rs"),
            ["fmt", "--file=a.rs"]
        );
        assert_eq!(args("date +%s %%", "{}", &all, "x"), ["date", "+%s", "%%"]);
        assert_eq!(args("{} -v", "{}", &all, "./run.sh"), ["./run.sh", "-v"]);
        assert_eq!(
            args("cat {}", "{}", &all, "my dir/a b"),
            ["cat", "my dir/a b"]
        );
        assert_eq!(
            args("du {@} -s", "{}", &all, "x"),
            ["du", "src/", "my dir", "-s"]
        );
        assert_eq!(
            args("jq '{}' @", "@", &all, "d.json"),
            ["jq", "{}", "d.json"]
        );
        assert_eq!(
            args("ls {@} @", "@", &all, "x"),
            ["ls", "src/", "my dir", "x"]
        );
        // Inserted paths are not substituted again
        assert_eq!(args("echo {}", "{}", &all, "a{}{@}"), ["echo", "a{}{@}"]);
    }

    #[test]
    fn uses_placeholder() {
        let new = |t: &str, p: &str| Command::new(t.into(), p.into(), &[]).unwrap();
        assert!(new("du {}", "{}").uses_placeholder());
        assert!(!new("du {@}", "{}").uses_placeholder());
        assert!(!new("du {@}", "@").uses_placeholder());
        assert!(!new("date +%s", "{}").uses_placeholder());
    }
}
