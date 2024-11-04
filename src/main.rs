mod util;

extern crate walkdir;
use walkdir::WalkDir;
use std::{env, fs, path::Path, process};
use util::{config::{self, Config, ConfigError, ParsedReturn, Query, QueryInfo}, init::{self, InitParams, OperationType, ProjectSetup}};
use util::query;
use crate::query::query;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut config : Config;
    config = Config::new_config();
    let mut query_info_to_pass = QueryInfo::new_query_info();
    let old_config : Option<Config> = if Path::new("config.toml").exists() {
        let config_result = Config::read_config("config.toml");
        let config = match config_result {
            Ok(config) => config,
            Err(error) => {
                eprintln!("Line {}: Problem opening the file: {}", line!(), error);
                std::process::exit(2);
            }
        };
        Some(config)
    } else {
        None
    };
    let operation_type;

    if &args.len() == &1usize {
        help();
    }

    let parsed_return = match &args[1][..] {
        "n" | "new" => {
            operation_type = OperationType::New;
            config::parse_args(args, false, &operation_type)
        },
        "u" | "update" => {
            operation_type = OperationType::Update;
            config::parse_args(args, true, &operation_type)
        },
        "q" | "query" => {
            operation_type = OperationType::Query;
            config::parse_args(args, true, &operation_type)
        },
        _ => {
            help();
            operation_type = OperationType::None;
            ParsedReturn::None
        },
    };

    match parsed_return {
        ParsedReturn::Config(returned_config) => {config = returned_config},
        ParsedReturn::Query(returned_query) => {query_info_to_pass = returned_query},
        ParsedReturn::None => {}, // Inaccessible
    }
    
    // dbg!(&config);
    if operation_type != OperationType::Query{
        setup(old_config, config, operation_type);
    } else{
        query(query_info_to_pass, config);
    }

    finish();
}

fn setup(old_config_option: Option<Config>, config: Config, op_type: OperationType){
    let mut old_config = Config::new_config();
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
            Ok(()) => {
                old_config.setup.name = setup.name.clone();
                // Regardless of whether old config exists or not (if it didn't it would get default values), edit the name attribute and then write the config to file.
                // UPDATE NAME VALUE IN CONFIG
                let write_config_result = Config::write_config(&old_config, "config.toml");
                match write_config_result {
                    Ok(file) => file,
                    Err(error) => {
                        eprintln!("Line {}: Problem opening the file: {}", line!(), error);
                        std::process::exit(1);
                    },
                };
            },
            Err(error) => {
                eprintln!("Line {}: Problem creating/editing files: {}", line!(), error);
                std::process::exit(3);
            },
        };
    }
    
    let mut paths: Vec<String> = Vec::new();
    
    {
        paths.push("/".to_string());
        for v in &config.file_structure.folders_list {
            let mut paths_to_append: Vec<String> = Vec::new();
            let next_path = String::from("/");
            paths_to_append.push(next_path);
            let mut current_parent_folder = v;
            let mut iterations = 0;
            let max_iterations = 100;
            // MAX ITERATIONS set to 100 (can be changed)
            while current_parent_folder.parent != 0 && iterations < max_iterations {
                // println!("{x}", x = current_parent_folder.name);
                // println!("{x}", x = current_parent_folder.parent);
                match current_parent_folder.name.as_str() {
                    "%days" => {
                        let mut new_paths_vector: Vec<String> = Vec::new();
                        for i in 1..setup.days + 1 {
                            let padded_number = format!("{:0>2}", i);
                            let folder_name = format!("/{}_DAY{}", padded_number, padded_number);
                            for path in &mut paths_to_append {
                                let mut new_path = path.clone();
                                new_path.insert_str(0, &folder_name);
                                new_paths_vector.push(new_path);
                            }
                        }
                        paths_to_append = new_paths_vector;
                    },
                    "%cams" => {
                        let mut new_paths_vector: Vec<String> = Vec::new();
                        for i in 1..setup.cameras + 1 {
                            let padded_number = format!("{:0>2}", i);
                            let folder_name = format!("/{x}_{y}_CAM", x = padded_number, y = num_to_char(i).to_string());
                            for path in &mut paths_to_append {
                                let mut new_path = path.clone();
                                new_path.insert_str(0, &folder_name);
                                new_paths_vector.push(new_path);
                            }
                        }
                        paths_to_append = new_paths_vector;
                    },
                    "%soundsources" => {
                        let mut new_paths_vector: Vec<String> = Vec::new();
                        for i in 1..setup.sound_sources + 1 {
                            let padded_number = format!("{:0>2}", i);
                            let folder_name = format!("/{x}_{y}_REC", x = padded_number, y = num_to_char(i).to_string());
                            for path in &mut paths_to_append {
                                let mut new_path = path.clone();
                                new_path.insert_str(0, &folder_name);
                                new_paths_vector.push(new_path);
                            }
                        }
                        paths_to_append = new_paths_vector;
                    },
                    _ => {
                        for path in &mut paths_to_append {
                            let current_folder_name = &current_parent_folder.name;
                            path.insert_str(0, &format!("/{}", current_folder_name));
                        }
                    },
                };
                current_parent_folder = &config.file_structure.folders_list.get(current_parent_folder.parent - 1).expect("Parent does not exist!");
                iterations += 1;
            }
            if max_iterations == iterations {
                panic!("Looped past max iterations!");
            }
            for path in &mut paths_to_append {
                path.insert_str(0, &setup.name);
            }
            paths.append(&mut paths_to_append);
        }
    }

    for path in &paths {
        if !Path::new(&path).exists() {
            match fs::create_dir(&path).map_err(|e| ConfigError::IoError(e)) {
                Ok(()) => {},
                Err(error) => {
                    eprintln!("Line {}: Problem creating file: {}", line!(), error);
                    std::process::exit(3);
                },
            }
            ;
        }
    }

    let write_config_result = Config::write_config(&config, "config.toml");
    match write_config_result {
        Ok(file) => file,
        Err(error) => {
            eprintln!("Line {}: Problem opening the file: {}", line!(), error);
            std::process::exit(1);
        },
    };

    // clean project will remove any empty folders in the project that are not defined.
    if config.setup.clean_project {
        for file in WalkDir::new(format!("./{}",setup.name)).into_iter().filter_map(|file| file.ok()) {
            if file.metadata().unwrap().is_dir() {
                if WalkDir::new(file.path()).into_iter().nth(1).is_none() {
                    let empty_directory = file.path().to_string_lossy().replace("\\", "/");
                    // if empty_directory.len() > 0 {
                    //     empty_directory.remove(0);  // remove first
                    // }
                    let mut should_exist = false;
                    for path in &paths {
                        let mut comparable_path = path.clone();
                        comparable_path.insert_str(0,"./");
                        comparable_path.pop();
                        // dbg!(&comparable_path);
                        // dbg!(&empty_directory);
                        if comparable_path == empty_directory {
                            should_exist = true;
                            break;
                        }
                    }
                    if !should_exist {
                        match fs::remove_dir(&empty_directory) {
                            Ok(()) => {
                                println!("Managed to delete empty directory {}", &empty_directory);
                            },
                            Err(error) => {
                                eprintln!("Line {}: Program failed to delete a folder it thought was empty!: {}", line!(), error);
                                std::process::exit(1);
                            }
                        }

                    }
                }
            }
        }
    }
}

// initializes the main folder, optionally renaming an older folder given the correct conditions.
fn initialize_main_folder(old_setup : &ProjectSetup, setup : &ProjectSetup, op_type: &OperationType, old_config_exists : bool) -> std::result::Result<(), ConfigError>{
    if old_config_exists && op_type == &OperationType::Update && Path::new(&old_setup.name).exists() && &old_setup.name != &setup.name {
        fs::rename(&old_setup.name, &setup.name).map_err(|e| ConfigError::IoError(e))?;
    } else {
        if !Path::new(&setup.name).exists(){
            fs::create_dir(&setup.name).map_err(|e| ConfigError::IoError(e))?;
        }
    }
    Ok(())
}

// initializes the main folder, renaming an older folder using its name.
fn initialize_main_folder_deadname(deadname: &String, setup: &ProjectSetup) -> std::result::Result<(), ConfigError>{
    if Path::new(&deadname).exists() {
        fs::rename(&deadname, &setup.name).map_err(|e| ConfigError::IoError(e))?;
    } else {
        if !Path::new(&setup.name).exists(){
            fs::create_dir(&setup.name).map_err(|e| ConfigError::IoError(e))?;
        }
    }
    Ok(())
}

fn num_to_char(num: usize) -> char {
    if num >= 1 && num <= 26 {
        (num as u8 + b'A' - 1) as char
    } else {
        let char: char = '_';
        char
    }
}

fn help() {
    println!("
nanopm v(0.1.0) || https://nanomotions.org || https://github.com/kaweepatinn1/nanopm

Usage: nanopm [OPERATION] [ARGUMENTS]

Operations: 
      
    new, n      | Initialize a new project in the current directory, creating a new config file from 
                  provided arguments, using defaults where missing.
    update, u   | Update the current config file based on provided arguments. Project Manager must already 
                  have been initialized.
    query, q    | Query the current project based on provided arguments. Project Manager must already have 
                  been initialized.

Arguments: 
          
    CONFIG ARGS | Works with either new/update operations:
    
        -n, --name <String>             | Names the project and its directory. When used with update, uses 
                                          the old config file to rename the old directory to the new name.
        -dn, --deadname <String>        | Looks for a directory with this name, updating it with the new 
                                          name provided (or default), making it the main project directory.
        -d, --days <Integer>            | Sets the amount of footage days the project should account for.
        -c, --cameras <Integer>         | Sets the amount of cameras the project should account for.
        -s, --sound-sources <Integer>   | Sets the amount of sound sources the project should account for.
        -cl, --clean                    | Cleans the project folder after initializing, deleting all empty 
                                          folders not defined by the program.

    QUERY ARGS | You can use ONE type of query at a time. Works with query operations only:
       
        GENERAL QUERY:

            -g, --general               | Creates a general query of various important project folders. 
                                          Can return sorted by size.
                -ss, --sort-size        | Sorts general query by size. Must be used after a general query.
                -sd, --sort-default     | Sorts general query by... its default order... Kind of redundant. 
                                          Must be used after a general query.
    
        PARTIAL QUERY:
    
            -r, --root                  | Queries the full project directory, as well as returning project 
                                          config values.
            -d, --day                   | Queries each day in RUSHES.
            -c, --camer                 | Queries each camera. Combines all days into one entry for each 
                                          camera, displays each day separately if --unique is used.
            -s, --sound-source          | Queries each sound source. Combines all days into one entry for 
                                          each source, displays each day separately if --unique is used.
                -u, --unique            | Stops nanopm from combining all days into one entry for --camera 
                                          and --sound-source. Unique folders are queried individually.
    
        FOLDER QUERY: 
    
            -f, --folder <String>       | Queries all folders with the name of the string. Can chain 
                                          multiple --folder calls to query multiple folder names at once.
    
        UNIVERSAL QUERY ARGS:
        -w, --write <String>            | Writes query result to file with the specified string path. 
                                          Uses timestamp for path instead if last parameter.
        -t, --timestamp                 | Adds a timestamp to the top of the query file, if written. 
                                          Does nothing if write is not specified. Sick!");
    finish();
}

fn finish() {
    process::exit(0);
}
