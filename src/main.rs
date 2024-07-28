use anyhow::{Context, Result};
use clap::Parser;

mod cli;
mod color;
mod gradiant;
mod image;
mod template;

use cli::{Cli, Commands};

//TODO: creating new colors when not enough primary colors found

//TODO: port templates somehow?

fn read_palette(path: &str) -> Result<color::Palette> {
    // let f = std::fs::File::open(path).context("failed to read file")?;
    let s = std::fs::read_to_string(path).context("failed to read file")?;
    let p: color::Palette = serde_json::from_str(&s).context("failed to deserialize")?;
    Ok(p)
}

fn main() -> Result<()> {
    // prepare image for kmeans by converting to Srgb
    let cli = Cli::parse();
    match cli.command {
        Commands::Generate(args) => args.generate(),
        Commands::Test => {
            let palette = read_palette("./colors.json")?;

            let tmpl = std::fs::read_to_string("./test.tmpl").unwrap();

            let mut t = template::Engine::new();
            let r = t.render(&tmpl, palette)?;
            println!("{r}");
            Ok(())
        }
    }
}
