use clap::Parser;
use std::env;

use crate::update;

#[derive(Parser)]
#[command(name = "climd")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "A fast, terminal-based Markdown viewer", long_about = None)]
pub struct Cli {
    #[arg(short, long, help = "Check for updates and install the latest version")]
    pub update: bool,
}

pub fn run_cli() -> bool {
    let cli = Cli::parse();

    if cli.update {
        if let Err(e) = update::run_update() {
            eprintln!("Update failed: {}", e);
        }
        return true;
    }

    false
}
