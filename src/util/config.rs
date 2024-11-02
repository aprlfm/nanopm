use config::{Config as ConfigLoader, File, FileFormat};
use serde::{Deserialize, Serialize};
//use std::fs;
use toml;
use crate::{init, InitParams, ProjectSetup};

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub setup: init::ProjectSetup,
}

impl Config {
    pub fn write_config(config: Config, file_path: &str) -> Config {
        let text = toml::to_string(&config).expect("Could not convert Config to TOML text!");
        std::fs::write(file_path, text).expect("Failed to write to file!");
        config
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
        project = Config::read_config("config.toml").expect("Could not read config!").setup;
    } else {
        project = ProjectSetup{
            name : String::from("Untitled"),
            days : 1,
            cameras : 1,
            sound_sources : 1,
        };
    }

    while args_to_process > 0 {

        arg_index += 1;
        let current_arg = &args[arg_index][..];
        
        if next_operation == InitParams::None {
            match current_arg {
                "-n" => next_operation = InitParams::ProjName,
                "--name" => next_operation = InitParams::ProjName,
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
                InitParams::Days => project.days = current_arg.parse().expect(&format!("Parameter after {} was not {}!", args[arg_index - 1], init::get_required_type(next_operation, true))[..]),
                InitParams::Cameras => project.cameras = current_arg.parse().expect(&format!("Parameter after {} was not {}!", args[arg_index - 1], init::get_required_type(next_operation, true))[..]),
                InitParams::SoundSources => project.sound_sources = current_arg.parse().expect(&format!("Parameter after {} was not {}!", args[arg_index - 1], init::get_required_type(next_operation, true))[..]),
                other => panic!("No defined instruction for processing \"{}\" (ERROR CODE: 1)", other.to_string()),
            }
            next_operation = InitParams::None
        }

        args_to_process -= 1;
        
        println!("{args_to_process} args left to process!");
    }

    if next_operation != InitParams::None {
        panic!("Parameter \"{}\" should be followed by {}!", args[arg_index], init::get_required_type(next_operation, true));
    }
    Config{setup : project}
}