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
        if template.trim().is_empty() {
            eprintln!("Empty command");
            return None;
        }
        let all_paths = paths.iter().map(|p| quote(p)).collect::<Vec<_>>().join(" ");
        let mut command = Self {
            template,
            placeholder,
            all_paths,
            uses_placeholder: false,
        };
        command.uses_placeholder = command.substitute("").1;
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

    /// Command line run by `sh -c`
    fn line(&self, current_path: Option<&Path>) -> String {
        self.substitute(&current_path.map(quote).unwrap_or_default())
            .0
    }

    pub fn run(&self, current_path: Option<&Path>, clean: bool) {
        // Prepare process
        let line = self.line(current_path);
        let mut process = std::process::Command::new("sh");
        process
            .arg("-c")
            .arg(&line)
            .stderr(Stdio::inherit())
            .stdout(Stdio::inherit());

        // Handle terminal
        if clean {
            execute!(std::io::stdout(), Clear(ClearType::All), MoveTo(0, 0)).ok(); // FUTURE : Error
            print_header(true, &line);
        } else {
            println!();
            print_header(false, &line);
        }

        // Spawn process
        let mut child = match process.spawn() {
            Ok(c) => c,
            Err(err) => {
                eprintln!("Failed to spawn process");
                eprintln!("Error : {err:?}");
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

fn print_header(with_right: bool, line: &str) {
    print!("{line}");
    let len_left = line.chars().count();

    if !with_right {
        println!();
        return;
    }

    let x = terminal::size().unwrap_or((0, 0)).0 as usize;

    // Right header
    let right = format!(
        "{}: {}",
        gethostname().to_string_lossy(),
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
    );
    let padding = x.saturating_sub(len_left + right.chars().count());
    println!("{}{right}", " ".repeat(padding));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(template: &str, placeholder: &str, paths: &[&str], current: &str) -> Vec<String> {
        let paths: Vec<PathBuf> = paths.iter().map(PathBuf::from).collect();
        let command = Command::new(template.into(), placeholder.into(), &paths).unwrap();
        // Words as sh sees them
        shlex::split(&command.line(Some(Path::new(current)))).unwrap()
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

    #[test]
    fn runs_through_sh() {
        let dir = std::env::temp_dir().join(format!("kaeo test {}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("a b.txt");
        std::fs::write(&file, "hello\n").unwrap();

        let template = "tr a-z A-Z < {} > {}.out && cat {}.out | wc -l && cat {}.out";
        let command = Command::new(template.into(), "{}".into(), &[]).unwrap();
        let output = std::process::Command::new("sh")
            .arg("-c")
            .arg(command.line(Some(&file)))
            .output()
            .unwrap();
        std::fs::remove_dir_all(&dir).unwrap();

        let stdout = String::from_utf8(output.stdout).unwrap();
        assert_eq!(
            stdout.split_whitespace().collect::<Vec<_>>(),
            ["1", "HELLO"]
        );
    }
}
