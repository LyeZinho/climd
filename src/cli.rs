use clap::{Parser, Subcommand};
use std::env;

use crate::update;

#[derive(Parser)]
#[command(name = "climd")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "A fast, terminal-based Markdown viewer", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    Update,
}

pub fn run_cli() -> bool {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Update) => {
            if let Err(e) = update::run_update() {
                eprintln!("Update failed: {}", e);
            }
            return true;
        }
        None => false,
    }
}
