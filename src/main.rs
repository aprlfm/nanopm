pub use config::Config;
use std::{env, fmt};

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
                _ => 
                    //panic!("Error in parsing: \"{other}\" is not a valid CLI argument!"),
                    next_operation = InitParams::None,
            }
        } else {
            match next_operation {
                InitParams::ProjName => project.name = String::from(current_arg),
                InitParams::Days => project.days = current_arg.parse().expect(&format!("Parameter after {} was not {}!", args[arg_index - 1], get_required_type(next_operation, true))[..]),
                InitParams::Cameras => project.cameras = current_arg.parse().expect(&format!("Parameter after {} was not {}!", args[arg_index - 1], get_required_type(next_operation, true))[..]),
                InitParams::SoundSources => project.sound_sources = current_arg.parse().expect(&format!("Parameter after {} was not {}!", args[arg_index - 1], get_required_type(next_operation, true))[..]),
                other => panic!("No defined instruction for processing \"{}\" (ERROR CODE: 1)", other.to_string()),
            }
            next_operation = InitParams::None
        }

        args_to_process -= 1;
        
        println!("{args_to_process} args left to process!");
    }

    if next_operation != InitParams::None {
        panic!("Parameter \"{}\" should be followed by {}!", args[arg_index], get_required_type(next_operation, true));
    }

    dbg!(&project);
}

fn get_required_type(operation : InitParams, readable : bool) -> String {
    if readable {
        match operation {
            InitParams::ProjName => String::from("a String"),
            InitParams::Days => String::from("an integer"),
            InitParams::Cameras => String::from("an integer"),
            InitParams::SoundSources => String::from("an integer"),
            _ => String::from("No type found for this parameter (ERROR CODE: 2)")
        }
    } else {
        match operation {
            InitParams::ProjName => String::from("String"),
            InitParams::Days => String::from("usize"),
            InitParams::Cameras => String::from("usize"),
            InitParams::SoundSources => String::from("usize"),
            _ => String::from("invalid")
        }
    }
}

#[derive(Eq, PartialEq, Debug)]
enum InitParams {
    None,
    ProjName,
    Days,
    Cameras,
    SoundSources,
}

impl InitParams {
    fn to_string(&self) -> String {
        match &self {
            InitParams::ProjName => String::from("ProjName"),
            InitParams::Days => String::from("Days"),
            InitParams::Cameras => String::from("Cameras"),
            InitParams::SoundSources => String::from("SoundSources"),
            other => panic!("Undefined InitParams variant \"{other:?}\" must be added to to_string() method (please report this) (ERROR CODE: 3)")
        }
    }
}

#[derive(Debug)]
struct ProjectSetup {
    name : String,
    days : usize,
    cameras : usize,
    sound_sources: usize,
}
