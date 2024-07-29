//! Cli Implementation

use std::io::Read;

use anyhow::{Context, Result};
use clap::{Args, Parser, Subcommand};

use crate::color::{Palette, PaletteColor};
use crate::gradiant::Gradiant;
use crate::image::RawImage;

#[derive(Debug, Parser)]
pub struct Cli {
    /// Wallgen Command
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Render palette definition to template
    Fill(FillArgs),
    /// Generate a new Color Palette
    Generate(GenerateArgs),
}

#[derive(Debug, Args)]
pub struct FillArgs {
    /// Template to render
    template: Option<String>,
    /// Palette definition used to render template
    #[clap(short, long, default_value = "colors.json")]
    palette: String,
    /// Output
    #[clap(short, long)]
    output: Option<String>,
}

impl FillArgs {
    fn read_palette(&self) -> Result<Palette> {
        log::info!("reading palette from {:?}", self.palette);
        let s = std::fs::read_to_string(&self.palette).context("file read failed")?;
        let p: Palette = serde_json::from_str(&s).context("deserialize failed")?;
        Ok(p)
    }
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
        let palette = self.read_palette().context("failed to load palette")?;
        let template = self.read_template().context("failed to read template")?;
        let mut engine = crate::template::Engine::new();
        let result = engine.render(&template, palette).context("render failed")?;
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
    palette: Gradiant,
    /// Output for Palette
    #[clap(short, long, default_value = "./colors.json")]
    output: String,
}

impl GenerateArgs {
    pub fn generate(self) -> Result<()> {
        let jpg = RawImage::new(&self.path, self.size).context("failed to load image")?;

        log::info!("calculating primary colors");
        let mut colors = jpg.kmeans(4);
        let mut sort_mode = "dark";

        colors.sort();
        if jpg.mean_luminocity() > 0.5 {
            sort_mode = "light";
            colors.reverse()
        }
        log::info!("determined color-mode: {sort_mode:?}");

        let mut gradiant = self.palette;
        if gradiant == Gradiant::Auto {
            if jpg.mean_saturation() < 0.12 {
                log::warn!("image saturation too low. reverting to mono palette");
                gradiant = Gradiant::Mono;
            } else {
                gradiant = Gradiant::Standard;
            }
        }

        log::info!("rendering text/accent colors");
        let mut palettes = vec![];
        for color in colors {
            let dark = color.luminocity() < 0.5;
            // determine text color
            let text_base = color.negative();
            let text_bright = if dark { 188 } else { 16 };
            let text_color = text_base.modulate(text_bright, 10, 100);
            // generate accent colors
            let mut accents = vec![];
            for (brightness, saturation) in gradiant.gradiant() {
                let sv = saturation as f32 / 100.0;
                let bv = brightness as f32 / 100.0;
                let accent = color.accent(sv, bv);
                accents.push(accent);
            }
            palettes.push(PaletteColor {
                primary: color,
                text: text_color,
                accents: accents.try_into().expect("not enough accents"),
            })
        }
        let palette = Palette {
            file: self.path.to_owned(),
            theme: sort_mode.to_owned(),
            gradiant,
            color3: palettes.pop().expect("not enough colors"),
            color2: palettes.pop().expect("not enough colors"),
            color0: palettes.pop().expect("not enough colors"),
            color1: palettes.pop().expect("not enough colors"),
        };
        let f = std::fs::File::create(&self.output).context("failed to create palette file")?;
        serde_json::to_writer(f, &palette).context("failed to serialize palette")
    }
}
