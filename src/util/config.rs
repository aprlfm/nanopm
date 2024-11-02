use config::{Config as ConfigLoader, File, FileFormat};
use serde::Deserialize;
//use std::fs;
use toml;

use super::init;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub setup: init::ProjectSetup,
}

impl Config {
    pub fn write_config(config: &Config, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let text = toml::to_string(&config.setup)?;
        std::fs::write(file_path, text)?;
        Ok(())
    }

    pub fn read_config(file_path: &str) -> Result<Self, config::ConfigError> {
        let config_loader = ConfigLoader::builder().add_source(File::new(file_path, FileFormat::Toml)).build()?;
        config_loader.try_deserialize()
    }
}