//! File Based Configuration

use std::{collections::HashMap, path::PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

use crate::color::Gradiant;

#[derive(Debug, Serialize, Deserialize)]
pub struct TemplateConfig {
    /// Template Source
    pub template: String,
    /// Template Render Destination
    pub target: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Default Gradiant for Palette generation
    pub gradiant: Gradiant,
    /// Template Configuration
    pub templates: HashMap<String, TemplateConfig>,
}

impl Config {
    /// Read Configuration from Path or Default
    pub fn new(path: Option<&String>) -> Result<Self> {
        // get filepath for configuration
        let path = match path {
            Some(path) => {
                let path = PathBuf::from(path);
                if !path.exists() {
                    return Err(anyhow!("no such config file: {path:?}"));
                }
                path
            }
            None => dirs::config_dir()
                .expect("failed to find config directory")
                .join("config.toml"),
        };
        // read configuration file or use default
        Ok(match path.exists() {
            true => {
                let cfg = std::fs::read_to_string(path).context("failed to read config file")?;
                toml::from_str(&cfg).context("failed to parse config")?
            }
            false => {
                log::warn!("config file missing. using default values");
                Config::default()
            }
        })
    }
}
