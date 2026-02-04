mod claude;
mod cli;
mod community;
mod error;
mod prompt;
mod storage;

use clap::Parser;
use cli::{App, Cli};

fn main() {
    let cli = Cli::parse();

    let result = App::new().and_then(|mut app| app.run(cli));

    if let Err(e) = result {
        eprintln!("{}: {}", "Error".red(), e);
        std::process::exit(1);
    }
}

// Re-export colored trait for main
use colored::Colorize;
