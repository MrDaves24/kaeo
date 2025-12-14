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
    let mut args = args::Args::parse();
    for path in args.path.iter_mut() {
        if let Some(canon) = check_path(path) {
            *path = canon;
        } else {
            return;
        }
    }
    let Some(command) = Command::new(args.command) else {
        return;
    };

    ctrlc::set_handler(|| {
        execute!(io::stdout(), terminal::LeaveAlternateScreen).ok(); // FUTURE : Error
        std::process::exit(0);
    })
    .ok(); // FUTURE : Error

    execute!(io::stdout(), terminal::EnterAlternateScreen).ok(); // FUTURE : Error
    match command.case() {
        command::Case::NoPath | command::Case::AllPaths | command::Case::AllPathsQuoted => {
            command.run(&args.path, None, true)
        }
        command::Case::OnePath => {
            command.run(&args.path, Some(&args.path[0]), true);
            for path in args.path.iter().skip(1) {
                command.run(&args.path, Some(path), false);
            }
        }
    }
    watch(command, args.path, args.recursive);
}
