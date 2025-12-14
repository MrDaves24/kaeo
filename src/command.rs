use crossterm::{
    cursor::MoveTo,
    execute,
    terminal::{self, Clear, ClearType},
};
use gethostname::gethostname;
use std::{
    ffi::OsString,
    path::{Path, PathBuf},
    process::Stdio,
};

#[derive(Clone)]
pub struct Command {
    case: Case,
    command: Vec<String>,
    placeholder_index: usize,
}
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Case {
    NoPath,
    OnePath,
    AllPaths,
    AllPathsQuoted,
    // FUTURE : OnePath AND AllPaths (or quoted) together ?
}

fn find_case(commands: &[String]) -> (Case, usize) {
    for (i, command) in commands.iter().enumerate() {
        if command.contains("{}") {
            return (Case::OnePath, i);
        } else if command.contains("%%") {
            return (Case::AllPathsQuoted, i);
        } else if command.contains("%") {
            return (Case::AllPaths, i);
        }
    }

    (Case::NoPath, 0)
}

impl Command {
    pub fn new(command: String) -> Option<Self> {
        let Some(command) = shlex::split(&command) else {
            eprintln!("Invalid command, cannot parse");
            return None;
        };
        if command.is_empty() {
            eprintln!("Empty command");
            return None;
        }
        let (case, placeholder_index) = find_case(&command);
        Some(Self {
            command,
            case,
            placeholder_index,
        })
    }
    pub fn run(&self, paths: &[PathBuf], current_path: Option<&Path>, clean: bool) {
        // Prepare process
        let mut process = std::process::Command::new(&self.command[0]);
        process.stderr(Stdio::inherit()).stdout(Stdio::inherit());
        if self.case == Case::NoPath {
            process.args(&self.command[1..]);
        } else {
            process.args(&self.command[1..self.placeholder_index]);
            match self.case {
                Case::NoPath => (),
                Case::OnePath => {
                    process.arg(current_path.unwrap());
                }
                Case::AllPaths => {
                    process.args(paths);
                }
                Case::AllPathsQuoted => {
                    let mut paths_joined = OsString::new();
                    let mut first = true;
                    for path in paths {
                        if first {
                            first = false;
                        } else {
                            paths_joined.push(" ");
                        }
                        paths_joined.push(path);
                    }
                    process.arg(paths_joined);
                }
            }
            process.args(&self.command[(self.placeholder_index + 1)..]);
        }

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
    pub fn case(&self) -> Case {
        self.case
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
