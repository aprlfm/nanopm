mod util;

pub use config::Config;
use std::env;
use util::{init::InitParams, init::ProjectSetup, init};

fn main() {
    let args: Vec<String> = env::args().collect();
    
    let mut args_to_process = args.len() - 1;
    let mut arg_index : usize = 0;
    let mut next_operation = InitParams::None;

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

    dbg!(&project);
}


