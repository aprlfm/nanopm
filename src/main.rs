mod util;

use std::{env, fmt::Result, fs, path::Path};
use serde::de::value::Error;
use util::{config::{self, new_config, Config, ConfigError}, init::{self, InitParams, OperationType, ProjectSetup}};

fn main() {
    let args: Vec<String> = env::args().collect();
    let config : Config;
    let old_config : Option<Config> = if Path::new("config.toml").exists() {
        let config_result = Config::read_config("config.toml");
        let config = match config_result {
            Ok(config) => config,
            Err(error) => {
                eprintln!("Problem opening the file: {}", error);
                std::process::exit(2);
            }
        };
        Some(config)
    } else {
        None
    };
    let mut operation_type = OperationType::None;

    config = match &args[1][..] {
        "new" => {
            operation_type = OperationType::New;
            config::parse_to_config(args, false)
        },
        "update" => {
            operation_type = OperationType::Update;
            config::parse_to_config(args, true)
        },
        _ => {
            println!("TODO: Help DOCUMENTATION");
            new_config()
        },
    };

    dbg!(&config);
    setup(&old_config, &config, &operation_type);

    let config_result = Config::write_config(&config, "config.toml");
    match config_result {
        Ok(file) => file,
        Err(error) => {
            eprintln!("Problem opening the file: {}", error);
            std::process::exit(1);
        },
    };
    
}

fn setup(old_config_option: &Option<Config>, config: &Config, op_type: &OperationType){
    let mut old_config = &new_config();
    let mut old_config_exists = false;
    match old_config_option {
        Some(config) => {old_config = config; old_config_exists = true},
        None => {},
    };

    let old_setup = &old_config.setup;
    let setup = &config.setup;

    {
        let main_folder_result : std::result::Result<(), ConfigError> = match &setup.deadname {
            Some(deadname) => initialize_main_folder_deadname(&deadname, &setup),
            None => initialize_main_folder(&old_setup, &setup, &op_type, old_config_exists),
        };

        match main_folder_result {
            Ok(()) => (),
            Err(error) => {
                eprintln!("Problem creating/editing files: {}", error);
                std::process::exit(3);
            },
        };
    }
    
}

// initializes the main folder, optionally renaming an older folder given the correct conditions.
fn initialize_main_folder(old_setup : &ProjectSetup, setup : &ProjectSetup, op_type: &OperationType, old_config_exists : bool) -> std::result::Result<(), ConfigError>{
    if old_config_exists && op_type == &OperationType::Update && Path::new(&old_setup.name).exists() && &old_setup.name != &setup.name {
        fs::rename(&old_setup.name, &setup.name).map_err(|e| ConfigError::IoError(e))?;
    } else {
        fs::create_dir(&setup.name).map_err(|e| ConfigError::IoError(e))?;
    }
    Ok(())
}

// initializes the main folder, renaming an older folder using its name.
fn initialize_main_folder_deadname(deadname: &String, setup: &ProjectSetup) -> std::result::Result<(), ConfigError>{
    if Path::new(&deadname).exists() {
        fs::rename(&deadname, &setup.name).map_err(|e| ConfigError::IoError(e))?;
    } else {
        fs::create_dir(&setup.name).map_err(|e| ConfigError::IoError(e))?;
    }
    Ok(())
}