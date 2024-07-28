//! Cli Implementation

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
    /// Generate a new Color Palette
    Generate(GenerateArgs),
    Test,
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

        log::debug!("running kmeans to find primary colors");
        let mut colors = jpg.kmeans(4);
        let mut sort_mode = "dark";

        colors.sort();
        if jpg.mean_luminocity() > 0.5 {
            sort_mode = "light";
            colors.reverse()
        }
        log::debug!("determined color-mode: {sort_mode:?}");

        let mut gradiant = self.palette;
        if gradiant == Gradiant::Auto {
            if jpg.mean_saturation() < 0.12 {
                log::warn!("image saturation too low. reverting to mono palette");
                gradiant = Gradiant::Mono;
            } else {
                gradiant = Gradiant::Standard;
            }
        }

        log::debug!("rendering text/accent colors");
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
