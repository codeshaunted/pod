mod commands;
mod config;
mod util;

use clap::Parser;
use commands::{Cli, CommandExecute, Commands};

fn main() {
    let args = Cli::parse();

    let result = match args.command {
        Commands::Init(args) => args.execute(),
        Commands::Split(args) => args.execute(),
        Commands::Gen(args) => args.execute(),
        Commands::Link(args) => args.execute(),
        Commands::PatchExe(args) => args.execute(),
    };

    if let Err(err) = result {
        eprintln!("error: {}", err);
        std::process::exit(1);
    }

    std::process::exit(0);
}
