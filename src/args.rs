use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(version, about, author, long_about = None)]
pub struct Args {
    #[arg(
        short,
        long,
        help = "Run command on the file that changed and not the parent (works with {} placeholder)"
    )]
    pub recursive: bool,

    // FUTURE : Filter change events
    // FUTURE : Don't clean terminal
    // FUTURE : Log to file
    // FUTURE : logs at all ?
    pub command: String,

    #[arg(required = true)]
    pub path: Vec<PathBuf>,
}
