use anyhow::{Context, Result};

mod color;
mod image;

const TEXT_DARK_BRIGHTNESS_MOD: u8 = 188;
const TEXT_LIGHT_BRIGHTNESS_MOD: u8 = 16;

const GRADIANT_STD: [(u8, u8); 8] = [
    (32, 50),
    (42, 46),
    (56, 39),
    (64, 38),
    (76, 37),
    (90, 33),
    (94, 29),
    (100, 20),
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

//standard: 32 50\n42 46\n49 40\n56 39\n64 38\n76 37\n90 33\n94 29\n100 20
//vibrant: 18 99\n32 97\n48 95\n55 90\n70 80\n80 70\n88 60\n94 40\n99 24
//mono:    10 0\n17 0\n24 0\n39 0\n51 0\n58 0\n72 0\n84 0\n99 0
//grey:    10 0\n17 0\n24 0\n39 0\n51 0\n58 0\n72 0\n84 0\n99 0

fn main() -> Result<()> {
    // prepare image for kmeans by converting to Srgb
    let jpg = image::RawImage::new("./japan-neo-4k.jpg", None).context("failed to load image")?;

    log::debug!("running kmeans to find primary colors");
    let mut colors = jpg.kmeans(4);
    let mut sort_mode = "dark";

    colors.sort();
    if jpg.mean_luminocity() > 0.5 {
        sort_mode = "light";
        colors.reverse()
    }
    log::debug!("determined color-mode: {sort_mode:?}");

    let mut gradiant: Vec<(u8, u8)> = GRADIANT_STD.to_vec();
    if jpg.mean_saturation() < 0.12 {
        gradiant = GRADIANT_MONO.to_vec();
    }

    for color in colors {
        let dark = color.luminocity() < 0.5;
        println!("primary={:?}", color.hex());
        // determine text color
        let text_base = color.negative();
        let text_bright = if dark {
            TEXT_DARK_BRIGHTNESS_MOD
        } else {
            TEXT_LIGHT_BRIGHTNESS_MOD
        };
        let text_color = text_base.modulate(text_bright, 10, 100);
        // generate accent colors
        println!("  text={:?}", text_color.hex());
        for (brightness, saturation) in gradiant.iter() {
            let sv = *saturation as f32 / 100.0;
            let bv = *brightness as f32 / 100.0;
            let accent = color.from_hue(sv, bv);
            println!("  color={:?}", accent.hex());
        }
    }

    Ok(())
}
