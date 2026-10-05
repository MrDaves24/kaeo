use clap::{Parser, builder::NonEmptyStringValueParser};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(version, about, author, long_about = None)]
pub struct Args {
    #[arg(
        short,
        long,
        help = "Give the file that changed to {} and not the watched path containing it"
    )]
    pub recursive: bool,

    #[arg(
        short = 'I',
        long,
        default_value = "{}",
        value_parser = NonEmptyStringValueParser::new(),
        help = "Placeholder for the changed path, if the command needs a literal {}"
    )]
    pub placeholder: String,

    // FUTURE : Filter change events
    // FUTURE : Don't clean terminal
    // FUTURE : Log to file
    // FUTURE : logs at all ?
    #[arg(help = "Command to run when a file or folder changes
Use {} to include the path that changed
Use {@} to include all watched paths, as separate arguments
Don't quote placeholders, paths are quoted when needed")]
    pub command: String,

    #[arg(required = true, help = "Paths to watch for changes")]
    pub path: Vec<PathBuf>,
}
