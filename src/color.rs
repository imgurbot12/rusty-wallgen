//! Color Operations

use anyhow::Result;
use palette::{FromColor, Hsl, Hsv, IntoColor, Lighten, Saturate, SetHue, ShiftHue, Srgb};

#[derive(Debug)]
pub struct Color(pub Srgb<f32>);

impl Color {
    fn hsl(&self) -> Hsl {
        let mut hsl: Hsl = self.0.into_color();
        hsl.set_hue((hsl.hue.into_inner() * 1000.0).round() / 1000.0);
        hsl
    }
}

impl Color {
    pub fn from_hex(hex: &str) -> Result<Self> {
        let i = u32::from_str_radix(hex.trim_start_matches("#"), 16)?;
        let rgb = Srgb::from(i);
        println!("color: {rgb:?}");
        Ok(Self(rgb.into_format()))
    }
    pub fn from_hue(&self, saturation: f32, value: f32) -> Color {
        let hsv: Hsv = self.0.into_color();
        let hue = (hsv.hue.into_inner() * 1000.0).round() / 1000.0;
        Hsv::new(hue, saturation, value).into_color()
    }
    pub fn rgb(&self) -> (u8, u8, u8) {
        self.0.into_format::<u8>().into_components()
    }
    pub fn hex(&self) -> String {
        let (r, g, b) = self.rgb();
        format!("#{r:02X}{g:02X}{b:02X}")
    }
    pub fn luminocity(&self) -> f32 {
        0.2126 * self.0.red + 0.7152 * self.0.green + 0.0722 * self.0.blue
    }
    pub fn negative(&self) -> Self {
        let (r, g, b) = self.rgb();
        let rgb: Srgb<u8> = Srgb::new(255 - r, 255 - g, 255 - b);
        Self(rgb.into_format())
    }
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

        // 1.88 => lighten(0.43)
        // 1.20 => lighten(0.10)
        // 0.16 => lighten(-0.84)
        // 0.50 => lighten(-0.50)
        // 1.30 => ligten(0.15)

        // 0.2  => desaturate(0.8)
        // 0.8  => desaturate(0.2)
        // 1.30 => saturate(0.62)
        // 1.80 => saturate(1.40)
        //
        // 1.80 => shift_hue(360.0 * 1.40) % 360.0
        // 1.40 => shift_hue(360.0 * 1.20) % 360.0
        // 1.20 => shift_hue(360.0 * 1.10) % 360.0
        // 0.8 => shift_hue(360.0 * 0.8) % 360.0

        // 10 * 150 = let vb = 0.02;
        // 10 * 188 = let bv = 0.035;
        // 100 * 150 = let bv = 0.32;
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
        self.hex().partial_cmp(&other.hex())
    }
}

impl Ord for Color {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.hex().cmp(&other.hex())
    }
}