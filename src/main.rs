mod util;

use std::env;
use util::{config::{self, new_config, Config}, init::{self, InitParams, ProjectSetup}};

fn main() {
    let args: Vec<String> = env::args().collect();

    let config : Config;
    match &args[1][..] {
        "init" => {
            let config_result = Config::write_config(config::new_config(), "config.toml");
            config = match config_result {
                Ok(file) => file,
                Err(error) => {
                    eprintln!("Problem opening the file: {}", error);
                    std::process::exit(1);
                },
            };
        },
        "new" => {
            let config_result = Config::write_config(config::parse_to_config(args, false), "config.toml");
            config = match config_result {
                Ok(file) => file,
                Err(error) => {
                    eprintln!("Problem opening the file: {}", error);
                    std::process::exit(1);
                },
            };
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
        },
        _ => {
            config = new_config();
            println!("TODO: Help DOCUMENTATION")
        },
    };
    dbg!(&config);
}