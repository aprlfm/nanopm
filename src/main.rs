mod util;

use std::{env, path::Path, fs};
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

    match &args[1][..] {
        "new" => {
            let config_result = Config::write_config(config::parse_to_config(args, false), "config.toml");
            config = match config_result {
                Ok(file) => file,
                Err(error) => {
                    eprintln!("Problem opening the file: {}", error);
                    std::process::exit(1);
                },
            };
            operation_type = OperationType::New;
        },
        "update" => {
            let config_result = Config::write_config(config::parse_to_config(args, true), "config.toml");
            config = match config_result {
                Ok(file) => file,
                Err(error) => {
                    eprintln!("Problem opening the file: {}", error);
                    std::process::exit(1);
                },
            };
            operation_type = OperationType::Update;
        },
        _ => {
            config = new_config();
            println!("TODO: Help DOCUMENTATION")
        },
    };

    dbg!(&config);
    setup(old_config, config, operation_type);
}

fn setup(old_config_option: Option<Config>, config: Config, op_type: OperationType){
    let mut old_config = new_config();
    let mut old_config_exists = false;
    match old_config_option {
        Some(config) => {old_config = config; old_config_exists = true},
        None => {},
    };

    let old_setup = old_config.setup;
    let setup = config.setup;

    match initialize_main_folder(&old_setup, &setup, &op_type, old_config_exists) {
        Ok(()) => (),
        Err(error) => {
            eprintln!("Problem creating/editing files: {}", error);
            std::process::exit(3);
        },
    };
    
    
}

fn initialize_main_folder(old_setup : &ProjectSetup, setup : &ProjectSetup, op_type: &OperationType, old_config_exists : bool) -> Result<(), config::ConfigError>{
    if old_config_exists && op_type == &OperationType::Update && Path::new(&old_setup.name).exists() && &old_setup.name != &setup.name {
        fs::rename(&old_setup.name, &setup.name).map_err(|e| ConfigError::IoError(e))?;
    } else {
        fs::create_dir(&setup.name).map_err(|e| ConfigError::IoError(e))?;
    }
    Ok(())
}