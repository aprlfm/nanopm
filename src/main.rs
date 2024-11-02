mod util;

use std::env;
use util::{config::{self, new_config, Config}, init::{self, InitParams, ProjectSetup}};

fn main() {
    let args: Vec<String> = env::args().collect();

    //println!("{args_to_process} args left to process!");
    let config : Config;
    match &args[1][..] {
        "init" => {
            config = Config::write_config(config::new_config(), "config.toml")
        },
        "new" => {
            config = Config::write_config(config::parse_to_config(args, false), "config.toml")
        },
        "update" => {
            config = Config::write_config(config::parse_to_config(args, true), "config.toml")
        },
        _ => {
            config = new_config();
            println!("TODO: Help DOCUMENTATION")
        },
    };
    dbg!(&config);
}