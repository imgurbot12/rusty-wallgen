//! Cli Implementation

use std::io::{Read, Write};
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Args, Parser, Subcommand};

use crate::color::{Gradiant, Palette};
use crate::config::Config;
use crate::image::RawImage;
use crate::template::Engine;

fn read_palette(palette: &str) -> Result<Palette> {
    log::info!("reading palette from {:?}", palette);
    let s = std::fs::read_to_string(palette).context("file read failed")?;
    let p: Palette = toml::from_str(&s).context("deserialize failed")?;
    Ok(p)
}

#[derive(Debug, Parser)]
pub struct Cli {
    /// Wallgen Command
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Generate a pallete from an image and apply
    Run(RunArgs),
    /// Render palette definition to template
    Fill(FillArgs),
    /// Generate a new Color Palette
    Generate(GenerateArgs),
}

#[derive(Debug, Args)]
pub struct RunArgs {
    image: String,
    /// Configuration source
    #[clap(short, long)]
    config: Option<String>,
    /// Default Gradiant to use
    #[clap(short, long)]
    gradiant: Option<Gradiant>,
    /// Shrink Image to Dimension before Analysis
    #[clap(short, long)]
    size: Option<u32>,
}

impl RunArgs {
    fn get_palette(&self, gradiant: Gradiant) -> Result<Palette> {
        match imghdr::from_file(&self.image).context("failed to detect file type")? {
            Some(_) => {
                let img =
                    RawImage::new(&self.image, self.size).context("failed to read base image")?;
                Ok(Palette::create(&img, gradiant))
            }
            None => read_palette(&self.image).context("failed to read palette file"),
        }
    }
    pub fn run(self) -> Result<()> {
        // read config
        let config = Config::new(self.config.as_ref())?;
        if config.templates.is_empty() {
            log::error!("no templates in config. no actions to complete!");
            return Err(anyhow::anyhow!("no templates in configuration"));
        }
        // read/generate palette
        let gradiant = self.gradiant.clone().unwrap_or(config.gradiant);
        let palette = self.get_palette(gradiant)?;
        // iterate templates and fill in palette information
        for (name, cfg) in config.templates {
            let target = shellexpand::tilde(&cfg.target).to_string();
            log::info!("writing template {name:?} {:?} => {target:?}", cfg.template);
            // generate directory for target
            if let Some(parent) = PathBuf::from(&target).parent() {
                if !parent.is_dir() {
                    std::fs::create_dir_all(parent)
                        .context(format!("{name:?} failed to make template target dir"))?;
                }
            }
            // render result to template target
            let mut engine = Engine::new();
            let base = std::fs::read_to_string(cfg.template).context("file read failed")?;
            let result = match engine.render(&base, &palette) {
                Ok(render) => std::fs::write(&target, render),
                Err(err) => Ok(log::warn!("{name:?} template render failed: {err:?}")),
            };
            if let Err(err) = result {
                log::warn!("{name:?} template write failed: {err:?}");
            }
        }
        Ok(())
    }
}

#[derive(Debug, Args)]
pub struct FillArgs {
    /// Template to render
    template: Option<String>,
    /// Palette definition used to render template
    #[clap(short, long, default_value = "colors.toml")]
    palette: String,
    /// Output
    #[clap(short, long)]
    output: Option<String>,
}

impl FillArgs {
    fn read_template(&self) -> Result<String> {
        if let Some(template) = self.template.as_ref() {
            log::info!("reading template from {template:?}");
            return Ok(std::fs::read_to_string(template).context("file read failed")?);
        }
        log::info!("reading template from stdin");
        let mut template = String::new();
        let mut stdin = std::io::stdin();
        stdin
            .read_to_string(&mut template)
            .context("failed to read stdin")?;
        Ok(template)
    }
    pub fn fill(self) -> Result<()> {
        let palette = read_palette(&self.palette).context("failed to load palette")?;
        let template = self.read_template().context("failed to read template")?;
        let mut engine = crate::template::Engine::new();
        let result = engine
            .render(&template, &palette)
            .context("render failed")?;
        match self.output {
            Some(output) => std::fs::write(output, result).context("failed to write output")?,
            None => println!("{result}"),
        }
        Ok(())
    }
}

#[derive(Debug, Args)]
pub struct GenerateArgs {
    /// Filepath of Wallpaper
    path: String,
    /// Shrink Image to Dimension before Analysis
    #[clap(short, long)]
    size: Option<u32>,
    /// Color Pallete Asignment
    #[clap(short, long, default_value = "auto")]
    gradiant: Gradiant,
    /// Output for Palette
    #[clap(short, long, default_value = "./colors.toml")]
    output: String,
}

impl GenerateArgs {
    pub fn generate(self) -> Result<()> {
        let img = RawImage::new(&self.path, self.size).context("failed to load image")?;
        let palette = Palette::create(&img, self.gradiant);
        let content = toml::to_string(&palette).context("failed to serialize palette")?;
        let mut f = std::fs::File::create(&self.output).context("failed to create palette file")?;
        write!(f, "{content}").context("failed to write to palette file")
    }
}
