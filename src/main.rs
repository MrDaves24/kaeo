use clap::Parser;
use crossterm::{execute, terminal};
use std::io;

use crate::{command::Command, helpers::check_path, watcher::watch};

pub mod args;
pub mod command;
pub mod helpers;
pub mod watcher;

fn main() {
    // Parse args
    let args = args::Args::parse();
    let mut canon = Vec::new();
    for path in &args.path {
        let Some(c) = check_path(path) else {
            return;
        };
        canon.push(c);
    }
    let Some(command) = Command::new(args.command, args.placeholder, &args.path) else {
        return;
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
    watch(command, args.path, canon, args.recursive);
}
