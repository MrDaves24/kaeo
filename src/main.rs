use clap::Parser;
use crossterm::{execute, terminal};
use std::{io, process::ExitCode};

use crate::{command::Command, helpers::check_path, watcher::watch};

pub mod args;
pub mod command;
pub mod helpers;
pub mod watcher;

fn main() -> ExitCode {
    // Parse args
    let args = args::Args::parse();
    let mut canon = Vec::new();
    for path in &args.path {
        let Some(c) = check_path(path) else {
            return ExitCode::FAILURE;
        };
        canon.push(c);
    }
    let Some(command) = Command::new(args.command, args.placeholder, &args.path) else {
        return ExitCode::FAILURE;
    };

    ctrlc::set_handler(|| {
        execute!(io::stdout(), terminal::LeaveAlternateScreen).ok(); // FUTURE : Error
        std::process::exit(0);
    })
    .ok(); // FUTURE : Error

    execute!(io::stdout(), terminal::EnterAlternateScreen).ok(); // FUTURE : Error
    if command.uses_placeholder() {
        for (i, path) in args.path.iter().enumerate() {
            command.run(Some(path), i == 0);
        }
    } else {
        command.run(None, true);
    }
    let error = watch(command, args.path, canon, args.recursive);
    // Print after leaving, the alternate screen would erase it
    execute!(io::stdout(), terminal::LeaveAlternateScreen).ok(); // FUTURE : Error
    eprintln!("{error}");
    ExitCode::FAILURE
}
