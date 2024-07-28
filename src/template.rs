//! Template Autofill

use anyhow::{Context, Result};
use minijinja::{context, Environment};

use crate::color::{Color, Palette};

fn parse_color(s: String) -> Result<Color, minijinja::Error> {
    Color::from_hex(&s).map_err(|_| {
        minijinja::Error::new(
            minijinja::ErrorKind::CannotDeserialize,
            format!("invalid color: {s:?}"),
        )
    })
}

fn rgb(s: String) -> Result<String, minijinja::Error> {
    let c = parse_color(s)?;
    let (r, g, b) = c.rgb();
    Ok(format!("rgb({r},{g},{b})"))
}

pub struct Engine<'a> {
    env: Environment<'a>,
}

impl<'a> Engine<'a> {
    pub fn new() -> Self {
        let mut env = Environment::new();
        env.add_filter("rgb", rgb);
        Self { env }
    }
    pub fn render(&mut self, template: &'a str, palette: Palette) -> Result<String> {
        self.env
            .add_template("main", &template)
            .context("failed to add template")?;
        let tmpl = self.env.get_template("main")?;
        Ok(tmpl.render(context!(
            file => palette.file,
            theme => palette.theme,
            gradiant => palette.gradiant,
            color0 => palette.color0,
            color1 => palette.color1,
            color2 => palette.color2,
            color3 => palette.color3,
        ))?)
    }
}
