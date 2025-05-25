mod util;

extern crate walkdir;
use std::{env, fs, path::Path, process};

use util::{
    config::{self, Config, ConfigError, ParsedReturn, Query, QueryInfo},
    init::{self, InitParams, OperationType, ProjectSetup},
    query,
};
use walkdir::WalkDir;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut config: Config;
    config = Config::new_config();
    let mut query_info_to_pass = QueryInfo::new_query_info();

    let old_config: Option<Config> = if Path::new("config.toml").exists() {
        let config_result = Config::read_config("config.toml");
        let config = match config_result {
            Ok(config) => config,
            Err(error) => {
                eprintln!("Line {}: Problem opening the file: {}", line!(), error);
                fs::rename("config.toml", "config_old.toml")
                    .map_err(|e| ConfigError::IoError(e))
                    .expect("Could not rename broken config");
                Config::new_config()
            }
        };
        Some(config)
    } else {
        None
    };

    if args.len() == 1 {
        help();
        return;
    }

    let operation_type = match &args[1][..] {
        "n" | "new" => OperationType::New,
        "u" | "update" => OperationType::Update,
        "q" | "query" => OperationType::Query,
        _ => {
            help();
            return;
        }
    };

    let parsed_return =
        config::parse_args(args, operation_type != OperationType::New, &operation_type);

    match parsed_return {
        ParsedReturn::Config(returned_config) => config = returned_config,
        ParsedReturn::Query(returned_query) => query_info_to_pass = returned_query,
        ParsedReturn::None => {}
    }

    if operation_type != OperationType::Query {
        if let Err(e) = setup(old_config, config, operation_type) {
            eprintln!("Setup failed: {}", e);
            process::exit(3);
        }
    } else {
        if let Err(e) = query::query(query_info_to_pass) {
            eprintln!("Query failed: {}", e);
            process::exit(4);
        }
    }

    finish();
}

fn setup(
    old_config_option: Option<Config>,
    config: Config,
    op_type: OperationType,
) -> Result<(), ConfigError> {
    let mut old_config = Config::new_config();
    let old_config_exists = if let Some(cfg) = old_config_option {
        old_config = cfg;
        true
    } else {
        false
    };

    config.validate()?;

    let old_setup = &old_config.setup;
    let setup = &config.setup;

    match &setup.deadname {
        Some(deadname) => initialize_main_folder_deadname(deadname, setup)?,
        None => initialize_main_folder(old_setup, setup, &op_type, old_config_exists)?,
    };

    old_config.setup.name = setup.name.clone();
    Config::write_config(&old_config, "config.toml")?;

    let paths = generate_folder_paths(&config)?;

    for path in &paths {
        if !Path::new(path).exists() {
            fs::create_dir_all(path).map_err(ConfigError::IoError)?;
        }
    }

    if config.setup.clean_project {
        clean_empty_directories(&config.setup.name, &paths)?;
    }

    Config::write_config(&config, "config.toml")?;

    let setup_to_print = toml::to_string(&config.setup)
        .map_err(|e| ConfigError::ParseError(format!("Failed to serialize config: {}", e)))?;

    println!("\n[Current Project Setup]\n{}", setup_to_print);
    Ok(())
}

fn generate_folder_paths(config: &Config) -> Result<Vec<String>, ConfigError> {
    let mut paths: Vec<String> = Vec::new();
    paths.push(config.setup.name.clone());

    for folder in &config.file_structure.folders_list {
        let folder_paths = build_folder_path(folder, config, &config.setup.name)?;
        paths.extend(folder_paths);
    }

    Ok(paths)
}

fn build_folder_path(
    folder: &util::config::Folder,
    config: &Config,
    project_name: &str,
) -> Result<Vec<String>, ConfigError> {
    let mut paths = Vec::new();

    match folder.name.as_str() {
        "%days" => {
            for i in 1..=config.setup.days {
                let folder_name = format!("{:02}_DAY{:02}", i, i);
                let full_path = if let Some(parent_id) = &folder.parent_id {
                    let parent_path = find_parent_path(parent_id, config, project_name)?;
                    format!("{}/{}", parent_path, folder_name)
                } else {
                    format!("{}/{}", project_name, folder_name)
                };
                paths.push(full_path);
            }
        }
        "%cams" => {
            for i in 1..=config.setup.cameras {
                let folder_name = format!("{:02}_{}_CAM", i, num_to_char(i));
                let full_path = if let Some(parent_id) = &folder.parent_id {
                    let parent_path = find_parent_path(parent_id, config, project_name)?;
                    format!("{}/{}", parent_path, folder_name)
                } else {
                    format!("{}/{}", project_name, folder_name)
                };
                paths.push(full_path);
            }
        }
        "%soundsources" => {
            for i in 1..=config.setup.sound_sources {
                let folder_name = format!("{:02}_{}_REC", i, num_to_char(i));
                let full_path = if let Some(parent_id) = &folder.parent_id {
                    let parent_path = find_parent_path(parent_id, config, project_name)?;
                    format!("{}/{}", parent_path, folder_name)
                } else {
                    format!("{}/{}", project_name, folder_name)
                };
                paths.push(full_path);
            }
        }
        _ => {
            let full_path = if let Some(parent_id) = &folder.parent_id {
                let parent_path = find_parent_path(parent_id, config, project_name)?;
                format!("{}/{}", parent_path, folder.name)
            } else {
                format!("{}/{}", project_name, folder.name)
            };
            paths.push(full_path);
        }
    }

    Ok(paths)
}

fn find_parent_path(
    parent_id: &str,
    config: &Config,
    project_name: &str,
) -> Result<String, ConfigError> {
    for folder in &config.file_structure.folders_list {
        if folder.id == parent_id {
            return Ok(if let Some(grandparent_id) = &folder.parent_id {
                let grandparent_path = find_parent_path(grandparent_id, config, project_name)?;
                format!("{}/{}", grandparent_path, folder.name)
            } else {
                format!("{}/{}", project_name, folder.name)
            });
        }
    }
    Err(ConfigError::ParseError(format!(
        "Parent folder with ID '{}' not found",
        parent_id
    )))
}

fn clean_empty_directories(project_name: &str, valid_paths: &[String]) -> Result<(), ConfigError> {
    let mut cleaned_this_pass = true;
    const MAX_ITERATIONS: i32 = 100;
    let mut iterations = 0;

    while cleaned_this_pass && iterations < MAX_ITERATIONS {
        cleaned_this_pass = false;
        iterations += 1;

        for entry in WalkDir::new(format!("./{}", project_name))
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.metadata().unwrap().is_dir() {
                if WalkDir::new(entry.path()).into_iter().nth(1).is_none() {
                    let empty_directory = entry.path().to_string_lossy().replace("\\", "/");
                    let mut should_exist = false;

                    for path in valid_paths {
                        let comparable_path = format!("./{}", path);
                        if comparable_path == empty_directory {
                            should_exist = true;
                            break;
                        }
                    }

                    if !should_exist {
                        fs::remove_dir(&empty_directory).map_err(ConfigError::IoError)?;
                        println!("Removed empty directory: {}", empty_directory);
                        cleaned_this_pass = true;
                    }
                }
            }
        }
    }

    if iterations >= MAX_ITERATIONS {
        return Err(ConfigError::ParseError(
            "Exceeded maximum cleanup iterations".to_string(),
        ));
    }

    Ok(())
}

fn initialize_main_folder(
    old_setup: &ProjectSetup,
    setup: &ProjectSetup,
    op_type: &OperationType,
    old_config_exists: bool,
) -> Result<(), ConfigError> {
    if old_config_exists
        && op_type == &OperationType::Update
        && Path::new(&old_setup.name).exists()
        && old_setup.name != setup.name
    {
        fs::rename(&old_setup.name, &setup.name).map_err(ConfigError::IoError)?;
    } else if !Path::new(&setup.name).exists() {
        fs::create_dir(&setup.name).map_err(ConfigError::IoError)?;
    }
    Ok(())
}

fn initialize_main_folder_deadname(
    deadname: &str,
    setup: &ProjectSetup,
) -> Result<(), ConfigError> {
    if Path::new(deadname).exists() {
        fs::rename(deadname, &setup.name).map_err(ConfigError::IoError)?;
    } else if !Path::new(&setup.name).exists() {
        fs::create_dir(&setup.name).map_err(ConfigError::IoError)?;
    }
    Ok(())
}

fn num_to_char(num: usize) -> char {
    if num >= 1 && num <= 26 {
        (num as u8 + b'A' - 1) as char
    } else {
        '_'
    }
}

fn help() {
    println!(
        "
nano project manager v(0.2.0) || https://nanomotions.org/scripts/nanopm || https://github.com/kaweepatinn1/nanopm
-----------------------------------------------------------------------------------------------------------------
Usage:
    nanopm [OPERATION] [ARGUMENTS]
-----------------------------------------------------------------------------------------------------------------
Operations:
    new, n      | Initialize a new project in the current directory, creating a new config file from
                  provided arguments, using defaults where missing.
    update, u   | Update the current config file based on provided arguments. Project Manager must already
                  have been initialized.
    query, q    | Query the current project based on provided arguments. Project Manager must already have
                  been initialized. Defaults to general query if no specific query type is provided.
-----------------------------------------------------------------------------------------------------------------
Arguments:
-----------------------------------------------------------------------------------------------------------------
    CONFIG ARGS | Works with either new/update operations:

        -n, --name <String>             | Names the project and its directory. When used with update, uses
                                          the old config file to rename the old directory to the new name.
        -dn, --deadname <String>        | Looks for a directory with this name, updating it with the new
                                          name provided if it exists, using it as the new project directory.
        -d, --days <Integer>            | Sets the amount of footage days the project should account for.
        -c, --cameras <Integer>         | Sets the amount of cameras the project should account for.
        -s, --sound-sources <Integer>   | Sets the amount of sound sources the project should account for.
        -cl, --clean                    | Cleans the project folder after initializing, deleting all empty
                                          folders not defined by the program.
-----------------------------------------------------------------------------------------------------------------
    QUERY ARGS | You can use ONE type of query at a time. Works with query operations only:

        GENERAL QUERY (default):

            -g, --general               | Creates a general query of various important project folders.
                                          Edit the list in config. Can return sorted by size.

        PARTIAL QUERY:

            -r, --root                  | Queries the full project directory, as well as returning project
                                          config values.
            -d, --days                  | Queries each day in RUSHES.
            -c, --cameras               | Queries each camera. Combines all days into one entry for each
                                          camera, displays each day separately if --unique is used.
            -s, --sound-sources         | Queries each sound source. Combines all days into one entry for
                                          each source, displays each day separately if --unique is used.
            -u, --unique                | Stops nanopm from combining all days into one entry for --cameras
                                          and --sound-sources. Unique folders are queried individually.

        FOLDER QUERY:

            -f, --folder <String>       | Queries all folders with the name of the string. Can chain
                                          multiple --folder calls to query multiple folder names at once.

        UNIVERSAL QUERY ARGS:
        -ss, --sort-size                | Sorts query results by size (largest first).
        -sd, --sort-default             | Sorts query results in default order.
        -w, --write <String>            | Writes query result to file with the specified string path.
                                          Uses timestamp for path instead if last parameter.
        -t, --timestamp                 | Adds a timestamp to the top of the query file, if written.
                                          Does nothing if write is not specified.
        -q, --quiet                     | Does not log missing folder errors into the console.
        -rt, --runtime                  | Includes runtime information in query results."
    );
    finish();
}

fn finish() {
    process::exit(0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_num_to_char() {
        assert_eq!(num_to_char(1), 'A');
        assert_eq!(num_to_char(26), 'Z');
        assert_eq!(num_to_char(0), '_');
        assert_eq!(num_to_char(27), '_');
    }
}
