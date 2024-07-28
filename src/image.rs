//! Image Operations and Analysis

use anyhow::Result;
use image::ImageReader;
use kmeans_colors::{get_kmeans_hamerly, Kmeans};
use palette::{cast::ComponentsAs, FromColor, Hsl, IntoColor, Srgb};

use crate::color::Color;

pub struct RawImage(Vec<Srgb<f32>>);

impl RawImage {
    pub fn new(path: &str) -> Result<Self> {
        // convert image to raw pixel buffer
        let img = ImageReader::open(path)?.with_guessed_format()?.decode()?;
        // .thumbnail(512, 512);
        let buf: Vec<u8> = img.into_rgb8().into_raw();
        // convert raw pixels into srgb objects
        let color_buffer: &[Srgb<u8>] = buf.components_as();
        let buffer = color_buffer.iter().map(|x| x.into_format()).collect();
        Ok(Self(buffer))
    }

    pub fn kmeans(&self, k: usize) -> Vec<Color> {
        let runs = 3;
        let max_iter = 20;
        let converge = 0.0025;
        let seed = 12345;
        let verbose = false;
        // run kmeans
        let mut result = Kmeans::new();
        for i in 0..runs {
            let run_result =
                get_kmeans_hamerly(k, max_iter, converge, verbose, &self.0, seed + i as u64);
            if run_result.score < result.score {
                result = run_result;
            }
        }
        // convert indexed colors back to hex-colors for output
        result
            .centroids
            .into_iter()
            .map(|c| Color::from_color(c))
            .collect()
    }

    pub fn mean_luminocity(&self) -> f32 {
        let pixels: Vec<Color> = self.0.iter().map(|c| Color::from_color(*c)).collect();
        let sum: f32 = pixels.iter().map(|c| c.luminocity()).sum();
        sum / pixels.len() as f32
    }

    pub fn mean_saturation(&self) -> f32 {
        let pixels: Vec<Hsl<_>> = self.0.iter().map(|c| (*c).into_color()).collect();
        let sum: f32 = pixels.iter().map(|c| c.saturation).sum();
        sum / pixels.len() as f32
    }
}
