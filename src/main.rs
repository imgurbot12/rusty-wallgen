use anyhow::Result;
use clap::Parser;

mod cli;
mod color;
mod gradiant;
mod image;
mod template;

use cli::{Cli, Commands};

//TODO: creating new colors when not enough primary colors found

//TODO: port templates somehow?

fn main() -> Result<()> {
    // prepare image for kmeans by converting to Srgb
    let cli = Cli::parse();
    match cli.command {
        Commands::Fill(args) => args.fill(),
        Commands::Generate(args) => args.generate(),
    }
}
