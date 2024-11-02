use config::{Config as ConfigLoader, File, FileFormat};
use serde::{Deserialize, Serialize};
use std::fmt;
use toml;
use crate::{init, InitParams, ProjectSetup};

use super::init::new_project_setup;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub setup: init::ProjectSetup,
}

impl Config {
    pub fn write_config(config: &Config, file_path: &str) -> Result<(), ConfigError> {
        let text = toml::to_string(&config)
            .map_err(|e| ConfigError::ParseError(format!("Failed to serialize config: {}", e)))?;
        
        std::fs::write(file_path, text)
            .map_err(|e| ConfigError::IoError(e))?;
            
        Ok(())
    }
    
    pub fn read_config(file_path: &str) -> Result<Self, config::ConfigError> {
        let config_loader = ConfigLoader::builder().add_source(File::new(file_path, FileFormat::Toml)).build()?;
        config_loader.try_deserialize()
    }
}

pub fn new_config() -> Config {
    Config {
        setup: init::new_project_setup(),
    }
}

pub fn parse_to_config(args : Vec<String>, load : bool) -> Config {
    let mut args_to_process = args.len() - 2;
    let mut arg_index : usize = 1;
    let mut next_operation = InitParams::None;

    let mut project: ProjectSetup;
    if load {
        let project_result = Config::read_config("config.toml");
        project = match project_result {
            Ok(config) => config.setup,
            Err(error) => {
                eprintln!("Problem opening the file: {}", error);
                std::process::exit(2);
            },
        };
    } else {
        project = new_project_setup()
    }

    while args_to_process > 0 {

        arg_index += 1;
        let current_arg = &args[arg_index][..];
        
        if next_operation == InitParams::None {
            match current_arg {
                "-n" => next_operation = InitParams::ProjName,
                "--name" => next_operation = InitParams::ProjName,
                "-dn" => next_operation = InitParams::DeadName,
                "--deadname" => next_operation = InitParams::DeadName,
                "-d" => next_operation = InitParams::Days,
                "--days" => next_operation = InitParams::Days,
                "-c" => next_operation = InitParams::Cameras,
                "--cameras" => next_operation = InitParams::Cameras,
                "-ss" => next_operation = InitParams::SoundSources,
                "--sound-sources" => next_operation = InitParams::SoundSources,
                other => panic!("Error in parsing: \"{other}\" is not a valid CLI argument!"),
            }
        } else {
            match next_operation {
                InitParams::ProjName => project.name = String::from(current_arg),
                InitParams::DeadName => project.deadname = Some(String::from(current_arg)),
                InitParams::Days => project.days = current_arg.parse().expect(&format!("Parameter after {} was not {}!", args[arg_index - 1], init::get_required_type(next_operation, true))[..]),
                InitParams::Cameras => project.cameras = current_arg.parse().expect(&format!("Parameter after {} was not {}!", args[arg_index - 1], init::get_required_type(next_operation, true))[..]),
                InitParams::SoundSources => project.sound_sources = current_arg.parse().expect(&format!("Parameter after {} was not {}!", args[arg_index - 1], init::get_required_type(next_operation, true))[..]),
                other => panic!("No defined instruction for processing \"{}\" (ERROR CODE: 1)", other.to_string()),
            }
            next_operation = InitParams::None
        }
        args_to_process -= 1;
    }

    if next_operation != InitParams::None {
        panic!("Parameter \"{}\" should be followed by {}!", args[arg_index], init::get_required_type(next_operation, true));
    }
    Config{setup : project}
}

#[derive(Debug)]
pub enum ConfigError {
    IoError(std::io::Error),
    ParseError(String),
    // InvalidArgument(String),
    // MissingParameter(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::IoError(e) => write!(f, "IO error! {}", e),
            ConfigError::ParseError(msg) => write!(f, "parsing error! {}", msg),
            // ConfigError::InvalidArgument(arg) => write!(f, "invalid argument! {}", arg),
            // ConfigError::MissingParameter(param) => write!(f, "missing parameter! {}", param),
        }
    }
}