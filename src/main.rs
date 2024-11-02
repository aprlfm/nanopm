pub use config::Config;
use std::{env, u8};

fn main() {
    let args: Vec<String> = env::args().collect();
    
    let mut args_to_process = args.len() - 1;
    let mut arg_index : usize = 0;
    let mut next_operation : u8 = InitParams::NONE;

    let mut project = ProjectSetup{
        name : String::from("Untitled"),
        days : 1,
        cameras : 1,
        sound_sources : 1,
    };

    println!("{args_to_process} args left to process!");
    while args_to_process > 0 {

        arg_index += 1;
        let current_arg = &args[arg_index][..];
        
        if next_operation == InitParams::NONE {
            match current_arg {
                "-n" => next_operation = InitParams::PROJ_NAME,
                "--name" => next_operation = InitParams::PROJ_NAME,
                "-d" => next_operation = InitParams::DAY_COUNT,
                "--days" => next_operation = InitParams::DAY_COUNT,
                "-c" => next_operation = InitParams::CAMERA_COUNT,
                "--cameras" => next_operation = InitParams::CAMERA_COUNT,
                "-ss" => next_operation = InitParams::SOUND_SOURCES,
                "--sound-sources" => next_operation = InitParams::SOUND_SOURCES,
                _ => 
                    //panic!("Error in parsing: \"{other}\" is not a valid CLI argument!"),
                    next_operation = InitParams::NONE,
            }
        } else {
            match next_operation {
                InitParams::PROJ_NAME => project.name = String::from(current_arg),
                InitParams::DAY_COUNT => project.days = current_arg.parse().expect(&format!("Parameter after {} was not {}!", args[arg_index - 1], get_required_type_readable(next_operation))[..]),
                InitParams::CAMERA_COUNT => project.cameras = current_arg.parse().expect(&format!("Parameter after {} was not {}!", args[arg_index - 1], get_required_type_readable(next_operation))[..]),
                InitParams::SOUND_SOURCES => project.sound_sources = current_arg.parse().expect(&format!("Parameter after {} was not {}!", args[arg_index - 1], get_required_type_readable(next_operation))[..]),
                other => panic!("\"next_operation\" somehow obtained invalid value of \"{other}\" (ERROR CODE: 1)"),
            }
            next_operation = InitParams::NONE
        }

        args_to_process -= 1;
        
        println!("{args_to_process} args left to process!");
    }

    if next_operation != InitParams::NONE {
        panic!("Parameter \"{}\" should be followed by {}!", args[arg_index], get_required_type_readable(next_operation));
    }

    dbg!(&project);
}

fn get_required_type(operation : u8) -> String {
    match operation {
        InitParams::PROJ_NAME => String::from("String"),
        InitParams::DAY_COUNT => String::from("usize"),
        InitParams::CAMERA_COUNT => String::from("usize"),
        InitParams::SOUND_SOURCES => String::from("usize"),
        _ => String::from("invalid")
    }
}

fn get_required_type_readable(operation : u8) -> String {
    match operation {
        InitParams::PROJ_NAME => String::from("a String"),
        InitParams::DAY_COUNT => String::from("an integer"),
        InitParams::CAMERA_COUNT => String::from("an integer"),
        InitParams::SOUND_SOURCES => String::from("an integer"),
        _ => String::from("No type found for this parameter (ERROR CODE: 2)")
    }
}

struct InitParams ();

impl InitParams {
    const NONE : u8 = 0;
    const PROJ_NAME : u8 = 1;
    const DAY_COUNT : u8 = 2;
    const CAMERA_COUNT : u8 = 3;
    const SOUND_SOURCES : u8 = 4;
}

#[derive(Debug)]
struct ProjectSetup {
    name : String,
    days : usize,
    cameras : usize,
    sound_sources: usize,
}
