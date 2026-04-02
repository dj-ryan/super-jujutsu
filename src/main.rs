mod ansi;
mod commands;
mod jj;
mod tui;

use clap::Parser;
use color_eyre::Result;
use std::process;

/// supjj — inline TUI flash card for Jujutsu VCS
#[derive(Parser)]
#[command(version, about)]
struct Cli {}

fn main() -> Result<()> {
    color_eyre::install()?;
    Cli::parse();

    if !jj::in_repo() {
        eprintln!("supjj: not inside a jj repository");
        process::exit(1);
    }

    let log_output = jj::log();
    let status_output = jj::status();

    let cmd = tui::run(&log_output, &status_output)?;

    if let Some(args) = cmd {
        if !args.is_empty() {
            let status = process::Command::new("jj")
                .args(args.split_whitespace())
                .status()?;
            process::exit(status.code().unwrap_or(1));
        }
    }

    Ok(())
}
