use config::{Config as ConfigLoader, File, FileFormat};
use serde::{Deserialize, Serialize};
use std::usize;
use std::fmt;
use toml;
use crate::{init, InitParams, ProjectSetup};
use crate::util::util::get_version;

use super::init::new_project_setup;
use super::init::OperationType;
use super::init::QueryParams;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub version : String,
    pub setup : init::ProjectSetup,
    pub file_structure : FileStructure,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FileStructure {
    pub folders_list: Vec<Folder>,
}

pub enum ParsedReturn {
    Config(Config),
    Query(Query, bool),
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum Query{
    None,
    General,
    Source,
    Folder(String),
    SubfoldersOf(String),
    Day(usize),
}

impl FileStructure {
    fn get_default_structure() -> Self {
        // vector inserts can NEVER reference a parent which has not been inserted yet.
        let mut vec: Vec<Folder> = Vec::new();
        vec.push(Folder{key : 1, parent : 0, name : String::from("%name")});                          // 01
        vec.push(Folder{key : 2, parent : 1, name : String::from("01_DOCUMENTATION")});               // 02
        vec.push(Folder{key : 3, parent : 1, name : String::from("02_RUSHES")});                      // 03
        vec.push(Folder{key : 4, parent : 1, name : String::from("03_EXTERNAL")});                    // 04
        vec.push(Folder{key : 5, parent : 1, name : String::from("04_PRE-RENDERS")});
        vec.push(Folder{key : 6, parent : 1, name : String::from("05_FINALS")});
        vec.push(Folder{key : 7, parent : 2, name : String::from("01_PRE-PRO")});
        vec.push(Folder{key : 8, parent : 2, name : String::from("02_PRODUCTION")});
        vec.push(Folder{key : 9, parent : 3, name : String::from("%days")});
        vec.push(Folder{key : 10, parent : 9, name : String::from("01_VIDEO")});
        vec.push(Folder{key : 11, parent : 9, name : String::from("02_AUDIO")});
        vec.push(Folder{key : 12, parent : 9, name : String::from("03_VO")});
        vec.push(Folder{key : 13, parent : 10, name : String::from("%cams")});
        vec.push(Folder{key : 14, parent : 11, name : String::from("%soundsources")});
        vec.push(Folder{key : 15, parent : 4, name : String::from("01_GRAPHICS")});
        vec.push(Folder{key : 16, parent : 4, name : String::from("02_IMAGES")});
        vec.push(Folder{key : 17, parent : 4, name : String::from("03_MUSIC")});
        vec.push(Folder{key : 18, parent : 4, name : String::from("04_SFX")});
        vec.push(Folder{key : 19, parent : 4, name : String::from("05_COMPS")});
        FileStructure{
            folders_list: vec,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
pub struct Folder {
    pub key :usize,
    pub parent : usize,
    pub name : String,
}

impl Config {
    pub fn write_config(config: &Config, file_path: &str) -> Result<(), ConfigError> {

        let mut text = toml::to_string(&config)
            .map_err(|e| ConfigError::ParseError(format!("Failed to serialize config: {}", e)))?;
        match &text.find("[[file") {
            Some(index) => {
                let finalindex = index.clone();
                text = format!("{}{}{}", &text[..finalindex], "# Edit below section at your own risk (the following changes file structure on future \"update\" calls)\n\n", &text[finalindex..]);
            }
            None => {}
        }
        
        std::fs::write(file_path, text)
            .map_err(|e| ConfigError::IoError(e))?;
            
        Ok(())
    }
    
    pub fn read_config(file_path: &str) -> Result<Self, config::ConfigError> {
        let config_loader = ConfigLoader::builder().add_source(File::new(file_path, FileFormat::Toml)).build()?;
        config_loader.try_deserialize()
    }
}

pub fn new_config() -> Config {
    Config {
        version : String::from("v1"),
        setup : init::new_project_setup(),
        file_structure : FileStructure::get_default_structure(),
    }
}

pub fn parse_args(args : Vec<String>, load : bool, op_type : &OperationType) -> ParsedReturn {
    let mut args_to_process = args.len() - 2;
    let mut arg_index : usize = 1;
    let mut next_init_param = InitParams::None;
    let mut next_query_param = QueryParams::None;

    let mut project: ProjectSetup;
    let structure: FileStructure;

    let mut query: Query = Query::None;
    let mut write_query = false;

    if load {
        let project_result = Config::read_config("config.toml");
        project = match project_result {
            Ok(config) => {structure = config.file_structure; config.setup},
            Err(error) => {
                eprintln!("Line {}: Problem opening the file: {}", line!(),error);
                std::process::exit(2);
            },
        };
    } else {
        project = new_project_setup();
        structure = FileStructure::get_default_structure();
    }

    while args_to_process > 0 && op_type != &OperationType::Query {

        arg_index += 1;
        let current_arg = &args[arg_index][..];
        
        if next_init_param == InitParams::None {
            match current_arg {
                "-n" => next_init_param = InitParams::ProjName,
                "--name" => next_init_param = InitParams::ProjName,
                "-dn" => next_init_param = InitParams::DeadName,
                "--deadname" => next_init_param = InitParams::DeadName,
                "-d" => next_init_param = InitParams::Days,
                "--days" => next_init_param = InitParams::Days,
                "-c" => next_init_param = InitParams::Cameras,
                "--cameras" => next_init_param = InitParams::Cameras,
                "-s" => next_init_param = InitParams::SoundSources,
                "--sound-sources" => next_init_param = InitParams::SoundSources,
                "-cl" => project.clean_project = true,
                "--clean" => project.clean_project = true,
                other => panic!("Error in parsing: \"{other}\" is not a valid CLI argument!"),
            }
        } else {
            match next_init_param {
                InitParams::ProjName => project.name = String::from(current_arg),
                InitParams::DeadName => project.deadname = Some(String::from(current_arg)),
                InitParams::Days => project.days = current_arg.parse().expect(&format!("Parameter after {} was not {}!", args[arg_index - 1], init::get_required_type_init(next_init_param, true))[..]),
                InitParams::Cameras => project.cameras = current_arg.parse().expect(&format!("Parameter after {} was not {}!", args[arg_index - 1], init::get_required_type_init(next_init_param, true))[..]),
                InitParams::SoundSources => project.sound_sources = current_arg.parse().expect(&format!("Parameter after {} was not {}!", args[arg_index - 1], init::get_required_type_init(next_init_param, true))[..]),
                InitParams::None => {}, // Should be inaccessible
            }
            next_init_param = InitParams::None
        }
        args_to_process -= 1;
    }

    if next_init_param != InitParams::None && op_type != &OperationType::Query {
        panic!("Parameter \"{}\" should be followed by {}!", args[arg_index], init::get_required_type_init(next_init_param, true));
    }

    while args_to_process > 0 && op_type == &OperationType::Query {
        arg_index += 1;
        let current_arg = &args[arg_index][..];


        if next_query_param == QueryParams::None {
            match current_arg {
                "-w" => write_query = true,
                "--write" => write_query = true,
                "-g" => {
                    if query == Query::None{
                        query = Query::General
                    } else{
                        panic!("Cannot have more than one query type!");
                    }
                },
                "--general" => {
                    if query == Query::None{
                        query = Query::General;
                    } else{
                        panic!("Cannot have more than one query type!");
                    }
                },
                "-s" => {
                    if query == Query::None{
                        query = Query::Source;
                    } else{
                        panic!("Cannot have more than one query type!");
                    }
                },
                "--source" => {
                    if query == Query::None{
                        query = Query::Source;
                    } else{
                        panic!("Cannot have more than one query type!");
                    }
                },
                "-f" => next_query_param = QueryParams::Folder,
                "--folder" => next_query_param = QueryParams::Folder,
                "-sf" => next_query_param = QueryParams::Subfolder,
                "--subfolders" => next_query_param = QueryParams::Subfolder,
                "-d" => next_query_param = QueryParams::Day,
                "--day" => next_query_param = QueryParams::Day,
                other => panic!("Error in parsing: \"{other}\" is not a valid CLI argument!"),
            }
        } else {
            match next_query_param {
                QueryParams::Folder => {
                    if query == Query::None{
                        query = Query::Folder(String::from(current_arg));
                    } else{
                        panic!("Cannot have more than one query type!");
                    }
                },
                QueryParams::Subfolder => {
                    if query == Query::None{
                        query = Query::SubfoldersOf(String::from(current_arg));
                    } else{
                        panic!("Cannot have more than one query type!");
                    }
                } ,
                QueryParams::Day => {
                    if query == Query::None{
                        query = Query::Folder(current_arg.parse().expect(&format!("Parameter after {} was not {}!", args[arg_index - 1], init::get_required_type_query(next_query_param, true))[..]),);
                    } else{
                        panic!("Cannot have more than one query type!");
                    }
                },
                QueryParams::None => {}, // Empty process (this should not be reachable)
            }
            next_query_param = QueryParams::None
        }
        args_to_process -= 1;
    }

    if next_query_param != QueryParams::None && op_type == &OperationType::Query {
        panic!("Parameter \"{}\" accepts values of {}!", args[arg_index], init::get_required_type_query(next_query_param, true));
    }
    if query == Query::None && op_type == &OperationType::Query {
        panic!("No query type specified!");
    }

    if op_type != &OperationType::Query {
        ParsedReturn::Config(
            Config{
                version : get_version(),
                setup : project,
                file_structure : structure,
            }
        )
    } else {
        ParsedReturn::Query(query, write_query)
    }

    
}

#[derive(Debug)]
pub enum ConfigError {
    IoError(std::io::Error),
    ParseError(String),
    // InvalidArgument(String),
    // MissingParameter(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::IoError(e) => write!(f, "IO error! {}", e),
            ConfigError::ParseError(msg) => write!(f, "parsing error! {}", msg),
            // ConfigError::InvalidArgument(arg) => write!(f, "invalid argument! {}", arg),
            // ConfigError::MissingParameter(param) => write!(f, "missing parameter! {}", param),
        }
    }
}