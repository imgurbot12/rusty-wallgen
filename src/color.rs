//! Color Operations

use std::{fmt::Display, str::FromStr};

use anyhow::{anyhow, Context, Result};
use palette::{FromColor, Hsl, Hsv, IntoColor, Saturate, SetHue, ShiftHue, Srgb};
use serde::{de::Error, Deserialize, Serialize};

use crate::image::RawImage;

const GRADIANT_STD: [(u8, u8); 9] = [
    (32, 50),
    (42, 46),
    (49, 40),
    (56, 39),
    (64, 38),
    (76, 37),
    (90, 33),
    (94, 29),
    (100, 20),
];
const GRADIANT_VIB: [(u8, u8); 9] = [
    (18, 99),
    (32, 97),
    (48, 95),
    (55, 90),
    (70, 80),
    (80, 70),
    (88, 60),
    (94, 40),
    (99, 24),
];
const GRADIANT_PASTEL: [(u8, u8); 9] = [
    (18, 99),
    (17, 66),
    (24, 29),
    (39, 41),
    (51, 37),
    (58, 34),
    (72, 30),
    (84, 26),
    (99, 22),
];
const GRADIANT_MONO: [(u8, u8); 9] = [
    (10, 0),
    (17, 0),
    (24, 0),
    (39, 0),
    (51, 0),
    (58, 0),
    (72, 0),
    (84, 0),
    (99, 0),
];

#[derive(Debug, Serialize, Deserialize)]
pub struct PaletteColor {
    pub primary: Color,
    pub text: Color,
    pub accents: [Color; 9],
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Palette {
    pub file: String,
    pub theme: String,
    pub gradiant: Gradiant,
    pub color0: PaletteColor,
    pub color1: PaletteColor,
    pub color2: PaletteColor,
    pub color3: PaletteColor,
}

impl Palette {
    /// Generate Palette of Colors with Specified Gradiant
    pub fn create(image: &RawImage, mut gradiant: Gradiant) -> Self {
        log::info!("calculating primary colors");
        let mut colors = image.kmeans(4);
        let mut sort_mode = "dark";

        colors.sort();
        if image.mean_luminocity() > 0.5 {
            sort_mode = "light";
            colors.reverse()
        }
        log::info!("determined color-mode: {sort_mode:?}");

        if gradiant == Gradiant::Auto {
            if image.mean_saturation() < 0.12 {
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
        let file = image.path().to_string_lossy();
        Self {
            file: file.as_ref().to_owned(),
            theme: sort_mode.to_owned(),
            gradiant,
            color3: palettes.pop().expect("not enough colors"),
            color2: palettes.pop().expect("not enough colors"),
            color1: palettes.pop().expect("not enough colors"),
            color0: palettes.pop().expect("not enough colors"),
        }
    }
}

#[derive(Debug)]
pub struct Color(pub Srgb<f32>);

impl Color {
    /// Generate rounded hsl equivalent to underlying Srgb value
    fn hsl(&self) -> Hsl {
        let mut hsl: Hsl = self.0.into_color();
        hsl.set_hue((hsl.hue.into_inner() * 1000.0).round() / 1000.0);
        hsl
    }
    /// Convert value from Hex string
    pub fn from_hex(s: &str) -> Result<Self> {
        let s = s.trim_start_matches("#");
        let i = u32::from_str_radix(s, 16).context("invalid hex value")?;
        let rgb = Srgb::from(i);
        Ok(Self(rgb.into_format()))
    }
    /// Convert value from rgb string
    pub fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        let rgb = Srgb::new(r, g, b);
        Self(rgb.into_format())
    }
}

impl Color {
    /// Generate new color with base hue and new Sautration/Brightness
    pub fn accent(&self, saturation: f32, value: f32) -> Color {
        let hsv: Hsv = self.0.into_color();
        let hue = (hsv.hue.into_inner() * 1000.0).round() / 1000.0;
        Hsv::new(hue, saturation, value).into_color()
    }
    /// Return separate red/green/blue values
    pub fn rgb(&self) -> (u8, u8, u8) {
        self.0.into_format::<u8>().into_components()
    }
    /// Generate color Hex value
    pub fn hex(&self) -> String {
        let (r, g, b) = self.rgb();
        format!("#{r:02X}{g:02X}{b:02X}")
    }
    /// Calculate color Luminocity
    pub fn luminocity(&self) -> f32 {
        0.2126 * self.0.red + 0.7152 * self.0.green + 0.0722 * self.0.blue
    }
    /// Generate negative inverse of color
    pub fn negative(&self) -> Self {
        let (r, g, b) = self.rgb();
        let rgb: Srgb<u8> = Srgb::new(255 - r, 255 - g, 255 - b);
        Self(rgb.into_format())
    }
    /// Generate a new color by modulating Brightness/Saturation/Hue percentages
    pub fn modulate(&self, brightness: u8, saturation: u8, hue: u8) -> Self {
        let color = Color::from_color(if brightness == 100 {
            self.0.clone()
        } else {
            // round(rgb * (BRIGHTNESS / 100.0))
            let value = brightness as f32 / 100.0;
            let mut color = self.0.clone();
            color.red *= value;
            color.green *= value;
            color.blue *= value;
            color
        });
        // calculate saturation/hue values
        let mut sv = saturation as f32 / 100.0;
        let mut hv = hue as f32 / 100.0;
        sv = if sv > 1.0 { (sv - 1.0) * 2.0 } else { sv - 1.0 };
        hv = (360.0 * hv) % 360.0;
        // finalize modulation
        let hsl: Hsl = color.hsl();
        let rgb = hsl.saturate(sv).shift_hue(hv).into_color();
        Self(rgb)
    }
}

impl<T> FromColor<T> for Color
where
    T: IntoColor<Srgb<f32>>,
{
    fn from_color(t: T) -> Self {
        Self(t.into_color())
    }
}

impl PartialEq for Color {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for Color {}

impl PartialOrd for Color {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.luminocity().partial_cmp(&other.luminocity())
    }
}

impl Ord for Color {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.luminocity()
            .partial_cmp(&other.luminocity())
            .expect("color ordering failed")
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.hex())
    }
}

impl FromStr for Color {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with("#") {
            return Ok(Self::from_hex(s)?);
        }
        if s.starts_with("rgb(") || s.starts_with("rgba(") {
            let (_, v) = s.split_once("(").expect("missing open paren");
            let (v, _) = v.split_once(")").context("missing rgb closing paren")?;
            let mut values = vec![];
            for i in v.split(",") {
                let i = u8::from_str(i).context("invalid color u8")?;
                values.push(i);
            }
            if values.len() < 3 {
                return Err(anyhow!("rgb has too few arguments"));
            }
            return Ok(Self::from_rgb(values[0], values[1], values[2]));
        }
        Err(anyhow!("invalid color: {s:?}"))
    }
}

impl Serialize for Color {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s: String = self.to_string();
        serializer.collect_str(&s)
    }
}

impl<'de> Deserialize<'de> for Color {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        Color::from_str(&s).map_err(D::Error::custom)
    }
}

/// Supported Color Gradiants used for Color Generation
#[derive(Debug, Clone, PartialEq)]
pub enum Gradiant {
    Auto,
    Standard,
    Vibrant,
    Pastel,
    Mono,
}

impl Gradiant {
    pub fn gradiant(&self) -> Vec<(u8, u8)> {
        match self {
            Self::Auto => GRADIANT_STD,
            Self::Standard => GRADIANT_STD,
            Self::Vibrant => GRADIANT_VIB,
            Self::Pastel => GRADIANT_PASTEL,
            Self::Mono => GRADIANT_MONO,
        }
        .to_vec()
    }
}

impl Default for Gradiant {
    fn default() -> Self {
        Self::Auto
    }
}

impl Display for Gradiant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Auto => "auto",
                Self::Standard => "standard",
                Self::Vibrant => "vibrant",
                Self::Pastel => "pastel",
                Self::Mono => "mono",
            }
        )
    }
}

impl FromStr for Gradiant {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "auto" => Ok(Self::Auto),
            "standard" => Ok(Self::Standard),
            "vibrant" => Ok(Self::Vibrant),
            "pastel" => Ok(Self::Pastel),
            "mono" => Ok(Self::Mono),
            _ => Err(format!("invalid palette: {s:?}")),
        }
    }
}

impl Serialize for Gradiant {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s: String = self.to_string();
        serializer.collect_str(&s)
    }
}

impl<'de> Deserialize<'de> for Gradiant {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        Gradiant::from_str(&s).map_err(D::Error::custom)
    }
}
