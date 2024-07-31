use anyhow::Result;
use clap::Parser;

mod cli;
mod color;
mod config;
mod image;
mod template;

use cli::{Cli, Commands};

//TODO: creating new colors when not enough primary colors found
//TODO: port templates somehow?

fn main() -> Result<()> {
    env_logger::init();

    let cli = Cli::parse();
    match cli.command {
        Commands::Run(args) => args.run(),
        Commands::Fill(args) => args.fill(),
        Commands::Generate(args) => args.generate(),
    }
}
