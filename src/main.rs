use anyhow::{Context, Result};
use palette::{FromColor, IntoColor};

mod color;
mod image;

//NOTE: running kmeans with k=1 is equal to finding average color of image

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

/*

primary=#1B548D
  text=#FFFFFF
  color=#293D52
  color=#3A526B
  color=#4B647D
  color=#57738F
  color=#6584A3
  color=#7A9EC2
  color=#9AC0E6
  color=#AACDF0
  color=#CCE6FF
fx_brightness (magick xc:#242D3C -colorspace gray -format %[fx:mean] info:)
primary=#242D3C
  text=#FFFFFF
  color=#293852
  color=#3A4C6B
  color=#4B5E7D
  color=#576C8F
  color=#657CA3
  color=#7A95C2
  color=#9AB6E6
  color=#AAC4F0
  color=#CCDFFF
fx_brightness (magick xc:#69375E -colorspace gray -format %[fx:mean] info:)
primary=#69375E
  text=#FFFFFF
  color=#522949
  color=#6B3A60
  color=#7D4B72
  color=#8F5783
  color=#A36596
  color=#C27AB2
  color=#E69AD5
  color=#F0AAE0
  color=#FFCCF4
fx_brightness (magick xc:#A97BC9 -colorspace gray -format %[fx:mean] info:)
primary=#A97BC9
  text=#0F100E
  color=#412952
  color=#573A6B
  color=#684B7D
  color=#78578F
  color=#8A65A3
  color=#A47AC2
  color=#C69AE6
  color=#D3AAF0
  color=#EACCFF
*/

/*
primary="#1B558C"
  text="#EEEEEA"
  color="#293E52"
  color="#3A536B"
  color="#57738F"
  color="#6585A3"
  color="#7A9FC2"
  color="#9AC0E6"
  color="#AACEF0"
  color="#CCE6FF"
primary="#252E3C"
  text="#FFFFFF"
  color="#293952"
  color="#3A4D6B"
  color="#576D8F"
  color="#657EA3"
  color="#7A96C2"
  color="#9AB8E6"
  color="#AAC6F0"
  color="#CCE0FF"
primary="#733862"
  text="#FFFFFF"
  color="#522946"
  color="#6B3A5D"
  color="#8F577F"
  color="#A36592"
  color="#C27AAE"
  color="#E69AD0"
  color="#F0AADC"
  color="#FFCCF1"
primary="#A881CE"
  text="#0E0F0D"
  color="#3D2952"
  color="#533A6B"
  color="#73578F"
  color="#8465A3"
  color="#9E7AC2"
  color="#C09AE6"
  color="#CDAAF0"
  color="#E6CCFF"
*/

//standard: 32 50\n42 46\n49 40\n56 39\n64 38\n76 37\n90 33\n94 29\n100 20
//vibrant: 18 99\n32 97\n48 95\n55 90\n70 80\n80 70\n88 60\n94 40\n99 24
//mono:    10 0\n17 0\n24 0\n39 0\n51 0\n58 0\n72 0\n84 0\n99 0
//grey:    10 0\n17 0\n24 0\n39 0\n51 0\n58 0\n72 0\n84 0\n99 0

//magick xc:'hsb(275.385,20%,100%)' -depth 8 -format %c histogram:info:
// 1: (234,204,255) #EACCFF srgb(234,204,255)

fn main() -> Result<()> {
    // prepare image for kmeans by converting to Srgb
    let jpg = image::RawImage::new("./japan-neo-4k.jpg").context("failed to load image")?;

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
