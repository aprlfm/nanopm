use core::mem::discriminant as tag;
use std::{fmt, usize};

use config::{Config as ConfigLoader, File, FileFormat};
use serde::{Deserialize, Serialize};
use toml;

use super::init::{OperationType, QueryParams, new_project_setup};
use crate::{InitParams, ProjectSetup, init, util::util::get_version};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub version: String,
    pub setup: init::ProjectSetup,
    pub file_structure: FileStructure,
    pub general_query_params: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FileStructure {
    pub folders_list: Vec<Folder>,
}

pub enum ParsedReturn {
    None,
    Config(Config),
    Query(QueryInfo),
}

pub struct QueryInfo {
    pub query: Query,
    pub settings: QuerySettings,
    pub config: Config,
}

impl QueryInfo {
    pub fn new_query_info() -> Self {
        QueryInfo {
            query: Query::None,
            settings: QuerySettings::default(),
            config: Config::new_config(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct QuerySettings {
    pub write: bool,
    pub output_name: Option<String>,
    pub record_timestamp: bool,
    pub unique_entries: bool,
    pub quiet: bool,
    pub include_runtime: bool,
}

impl QuerySettings {
    pub fn default() -> Self {
        QuerySettings {
            write: false,
            output_name: None,
            record_timestamp: false,
            unique_entries: false,
            quiet: false,
            include_runtime: false,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum Query {
    None,
    General(SortType),
    Partial(Vec<QueryType>, SortType),
    Folder(Vec<String>, SortType),
}

impl PartialEq<Self> for Query {
    fn eq(&self, rhs: &Self) -> bool {
        tag(self) == tag(rhs)
    }
}

impl Query {
    pub fn get_sort_type(&self) -> &SortType {
        match self {
            Query::General(sort_type) => sort_type,
            Query::Partial(_, sort_type) => sort_type,
            Query::Folder(_, sort_type) => sort_type,
            Query::None => &SortType::ByDefaultOrder,
        }
    }

    pub fn get_default_general_query() -> Vec<String> {
        vec![
            String::from("01_DOCUMENTATION"),
            String::from("01_VIDEO"),
            String::from("02_AUDIO"),
            String::from("03_VO"),
            String::from("01_GRAPHICS"),
            String::from("02_IMAGES"),
            String::from("03_MUSIC"),
            String::from("04_SFX"),
            String::from("05_COMPS"),
            String::from("04_PRE-RENDERS"),
            String::from("05_FINALS"),
        ]
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub enum SortType {
    None,
    BySize,
    ByDefaultOrder,
}

impl SortType {
    pub fn default_sort_type() -> Self {
        SortType::ByDefaultOrder
    }

    pub fn any() -> Self {
        SortType::None
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub enum QueryType {
    Root,
    Days,
    Cams,
    Sound,
}

impl FileStructure {
    fn get_default_structure() -> Self {
        let mut folders = Vec::new();

        folders.push(Folder::new("doc", None, "01_DOCUMENTATION"));
        folders.push(Folder::new("rushes", None, "02_RUSHES"));
        folders.push(Folder::new("external", None, "03_EXTERNAL"));
        folders.push(Folder::new("prerenders", None, "04_PRE-RENDERS"));
        folders.push(Folder::new("finals", None, "05_FINALS"));

        folders.push(Folder::new("prepro", Some("doc"), "01_PRE-PRO"));
        folders.push(Folder::new("production", Some("doc"), "02_PRODUCTION"));

        folders.push(Folder::new("days", Some("rushes"), "%days"));
        folders.push(Folder::new("video", Some("days"), "01_VIDEO"));
        folders.push(Folder::new("audio", Some("days"), "02_AUDIO"));
        folders.push(Folder::new("vo", Some("days"), "03_VO"));
        folders.push(Folder::new("cams", Some("video"), "%cams"));
        folders.push(Folder::new("soundsources", Some("audio"), "%soundsources"));

        folders.push(Folder::new("graphics", Some("external"), "01_GRAPHICS"));
        folders.push(Folder::new("images", Some("external"), "02_IMAGES"));
        folders.push(Folder::new("music", Some("external"), "03_MUSIC"));
        folders.push(Folder::new("sfx", Some("external"), "04_SFX"));
        folders.push(Folder::new("comps", Some("external"), "05_COMPS"));

        FileStructure {
            folders_list: folders,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone)]
pub struct Folder {
    pub id: String,
    pub parent_id: Option<String>,
    pub name: String,
}

impl Folder {
    pub fn new(id: &str, parent_id: Option<&str>, name: &str) -> Self {
        Folder {
            id: id.to_string(),
            parent_id: parent_id.map(|s| s.to_string()),
            name: name.to_string(),
        }
    }
}

impl Config {
    pub fn write_config(config: &Config, file_path: &str) -> Result<(), ConfigError> {
        let mut text = toml::to_string(config)
            .map_err(|e| ConfigError::ParseError(format!("Failed to serialize config: {}", e)))?;

        if let Some(index) = text.find("[[file_structure.folders_list]]") {
            text = format!(
                "{}{}{}",
                &text[..index],
                "# Edit below section at your own risk (the following changes file structure on \
                 \n# future \"update\" calls. Will not move files.)\n\n",
                &text[index..]
            );
        }

        std::fs::write(file_path, text).map_err(ConfigError::IoError)?;
        Ok(())
    }

    pub fn read_config(file_path: &str) -> Result<Self, config::ConfigError> {
        let config_loader = ConfigLoader::builder()
            .add_source(File::new(file_path, FileFormat::Toml))
            .build()?;
        config_loader.try_deserialize()
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.setup.name.trim().is_empty() {
            return Err(ConfigError::ParseError(
                "Project name cannot be empty".to_string(),
            ));
        }
        if self.setup.days == 0 {
            return Err(ConfigError::ParseError(
                "Number of days must be greater than 0".to_string(),
            ));
        }
        if self.setup.cameras == 0 {
            return Err(ConfigError::ParseError(
                "Number of cameras must be greater than 0".to_string(),
            ));
        }
        if self.setup.sound_sources == 0 {
            return Err(ConfigError::ParseError(
                "Number of sound sources must be greater than 0".to_string(),
            ));
        }
        Ok(())
    }
}

impl Config {
    pub fn new_config() -> Self {
        Config {
            version: get_version(),
            setup: new_project_setup(),
            file_structure: FileStructure::get_default_structure(),
            general_query_params: Query::get_default_general_query(),
        }
    }
}

pub fn parse_args(args: Vec<String>, load: bool, op_type: &OperationType) -> ParsedReturn {
    let mut args_to_process = args.len().saturating_sub(2);
    let mut arg_index: usize = 1;
    let mut next_init_param = InitParams::None;
    let mut next_query_param = QueryParams::None;

    let mut project: ProjectSetup;
    let structure: FileStructure;
    let general_query_params: Vec<String>;

    let mut query = Query::None;
    let mut query_settings = QuerySettings::default();

    if load {
        let project_result = Config::read_config("config.toml");
        project = match project_result {
            Ok(config) => {
                structure = config.file_structure;
                general_query_params = config.general_query_params;
                config.setup
            }
            Err(error) => {
                eprintln!(
                    "Line {}: Problem opening the file: {}
                    \nIf the file was not found, consider reinitializing the project with nanopm \
                     new (optionally add the argument -dn to consider an existing directory as \
                     the project directory).",
                    line!(),
                    error
                );
                std::process::exit(2);
            }
        };
    } else {
        project = new_project_setup();
        structure = FileStructure::get_default_structure();
        general_query_params = Query::get_default_general_query();
    }

    while args_to_process > 0 && op_type != &OperationType::Query {
        let print_query = if load { "Updated" } else { "Set" };

        arg_index += 1;
        let current_arg = &args[arg_index];

        if next_init_param == InitParams::None {
            match current_arg.as_str() {
                "-n" | "--name" => next_init_param = InitParams::ProjName,
                "-dn" | "--deadname" => next_init_param = InitParams::DeadName,
                "-d" | "--days" => next_init_param = InitParams::Days,
                "-c" | "--cameras" => next_init_param = InitParams::Cameras,
                "-s" | "--sound-sources" => next_init_param = InitParams::SoundSources,
                "-cl" | "--clean" => {
                    println!("Cleaning empty folders that are undefined!");
                    project.clean_project = true;
                }
                other => {
                    eprintln!(
                        "Error in parsing: \"{}\" is not a valid CLI argument!",
                        other
                    );
                    std::process::exit(1);
                }
            }
        } else {
            match next_init_param {
                InitParams::ProjName => {
                    project.name = String::from(current_arg);
                    println!("{} project name to {}", print_query, current_arg);
                }
                InitParams::DeadName => {
                    project.deadname = Some(String::from(current_arg));
                    println!("{} deadname to: {}", print_query, current_arg);
                }
                InitParams::Days => {
                    project.days =
                        parse_positive_integer(current_arg, "days").unwrap_or_else(|e| {
                            eprintln!("{}", e);
                            std::process::exit(1);
                        });
                    println!("{} project days: {}", print_query, current_arg);
                }
                InitParams::Cameras => {
                    project.cameras = parse_positive_integer(current_arg, "cameras")
                        .unwrap_or_else(|e| {
                            eprintln!("{}", e);
                            std::process::exit(1);
                        });
                    println!("{} cameras to: {}", print_query, current_arg);
                }
                InitParams::SoundSources => {
                    project.sound_sources = parse_positive_integer(current_arg, "sound sources")
                        .unwrap_or_else(|e| {
                            eprintln!("{}", e);
                            std::process::exit(1);
                        });
                    println!("{} sound sources to: {}", print_query, current_arg);
                }
                InitParams::None => {}
            }
            next_init_param = InitParams::None;
        }
        args_to_process = args_to_process.saturating_sub(1);
    }

    if next_init_param != InitParams::None && op_type != &OperationType::Query {
        eprintln!(
            "Parameter \"{}\" should be followed by {}!",
            args[arg_index],
            init::get_required_type_init(next_init_param, true)
        );
        std::process::exit(1);
    }

    let mut queries_to_run: Vec<QueryType> = Vec::new();
    let mut folders_to_search: Vec<String> = Vec::new();

    while args_to_process > 0 && op_type == &OperationType::Query {
        arg_index += 1;
        let current_arg = &args[arg_index];

        if next_query_param == QueryParams::None {
            match current_arg.as_str() {
                "-w" | "--write" => {
                    query_settings.write = true;
                    next_query_param = QueryParams::OutputDir;
                }
                "-t" | "--timestamp" => query_settings.record_timestamp = true,
                "-u" | "--unique" => query_settings.unique_entries = true,
                "-q" | "--quiet" => query_settings.quiet = true,
                "-rt" | "--runtime" => query_settings.include_runtime = true,
                "-g" | "--general" => {
                    if query == Query::None {
                        query = Query::General(SortType::default_sort_type());
                    } else {
                        eprintln!("Cannot have more than one query type!");
                        std::process::exit(1);
                    }
                }
                "-ss" | "--sort-size" => {
                    query = match query {
                        Query::General(_) => Query::General(SortType::BySize),
                        Query::Partial(queries, _) => Query::Partial(queries, SortType::BySize),
                        Query::Folder(folders, _) => Query::Folder(folders, SortType::BySize),
                        Query::None => {
                            eprintln!("Please specify a query type before specifying a sort type!");
                            std::process::exit(1);
                        }
                    };
                }
                "-sd" | "--sort-default" => {
                    query = match query {
                        Query::General(_) => Query::General(SortType::ByDefaultOrder),
                        Query::Partial(queries, _) => {
                            Query::Partial(queries, SortType::ByDefaultOrder)
                        }
                        Query::Folder(folders, _) => {
                            Query::Folder(folders, SortType::ByDefaultOrder)
                        }
                        Query::None => {
                            eprintln!("Please specify a query type before specifying a sort type!");
                            std::process::exit(1);
                        }
                    };
                }
                "-r" | "--root" => {
                    if query == Query::None || matches!(&query, Query::Partial(_, _)) {
                        queries_to_run.push(QueryType::Root);
                        query =
                            Query::Partial(queries_to_run.clone(), query.get_sort_type().clone());
                    } else {
                        eprintln!("Cannot have more than one query type!");
                        std::process::exit(1);
                    }
                }
                "-d" | "--days" => {
                    if query == Query::None || matches!(&query, Query::Partial(_, _)) {
                        queries_to_run.push(QueryType::Days);
                        query =
                            Query::Partial(queries_to_run.clone(), query.get_sort_type().clone());
                    } else {
                        eprintln!("Cannot have more than one query type!");
                        std::process::exit(1);
                    }
                }
                "-c" | "--cameras" => {
                    if query == Query::None || matches!(&query, Query::Partial(_, _)) {
                        queries_to_run.push(QueryType::Cams);
                        query =
                            Query::Partial(queries_to_run.clone(), query.get_sort_type().clone());
                    } else {
                        eprintln!("Cannot have more than one query type!");
                        std::process::exit(1);
                    }
                }
                "-s" | "--sound-sources" => {
                    if query == Query::None || matches!(&query, Query::Partial(_, _)) {
                        queries_to_run.push(QueryType::Sound);
                        query =
                            Query::Partial(queries_to_run.clone(), query.get_sort_type().clone());
                    } else {
                        eprintln!("Cannot have more than one query type!");
                        std::process::exit(1);
                    }
                }
                "-f" | "--folder" => next_query_param = QueryParams::Folder,
                other => {
                    eprintln!(
                        "Error in parsing: \"{}\" is not a valid CLI argument!",
                        other
                    );
                    std::process::exit(1);
                }
            }
        } else {
            match next_query_param {
                QueryParams::Folder => {
                    if query == Query::None || matches!(&query, Query::Folder(_, _)) {
                        folders_to_search.push(String::from(current_arg));
                        query =
                            Query::Folder(folders_to_search.clone(), query.get_sort_type().clone());
                    } else {
                        eprintln!("Cannot have more than one query type!");
                        std::process::exit(1);
                    }
                }
                QueryParams::OutputDir => {
                    query_settings.output_name = Some(String::from(current_arg));
                }
                QueryParams::None => {}
            }
            next_query_param = QueryParams::None;
        }
        args_to_process = args_to_process.saturating_sub(1);
    }

    if next_query_param != QueryParams::None
        && op_type == &OperationType::Query
        && next_query_param != QueryParams::OutputDir
    {
        eprintln!(
            "Parameter \"{}\" accepts values of {}!",
            args[arg_index],
            init::get_required_type_query(next_query_param, true)
        );
        std::process::exit(1);
    }

    if query == Query::None && op_type == &OperationType::Query {
        query = Query::General(SortType::ByDefaultOrder);
        if !query_settings.quiet {
            println!("No query type specified, defaulting to general query.");
        }
    }

    if op_type != &OperationType::Query {
        ParsedReturn::Config(Config {
            version: get_version(),
            setup: project,
            file_structure: structure,
            general_query_params,
        })
    } else {
        ParsedReturn::Query(QueryInfo {
            query,
            settings: query_settings,
            config: Config {
                version: get_version(),
                setup: project,
                file_structure: structure,
                general_query_params,
            },
        })
    }
}

fn parse_positive_integer(arg: &str, param_name: &str) -> Result<usize, String> {
    match arg.parse::<usize>() {
        Ok(0) => Err(format!("{} must be greater than 0", param_name)),
        Ok(n) => Ok(n),
        Err(_) => Err(format!("{} must be a positive integer", param_name)),
    }
}

#[derive(Debug)]
pub enum ConfigError {
    IoError(std::io::Error),
    ParseError(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::IoError(e) => write!(f, "IO error: {}", e),
            ConfigError::ParseError(msg) => write!(f, "Parsing error: {}", msg),
        }
    }
}

impl std::error::Error for ConfigError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_positive_integer() {
        assert_eq!(parse_positive_integer("5", "test").unwrap(), 5);
        assert!(parse_positive_integer("0", "test").is_err());
        assert!(parse_positive_integer("-1", "test").is_err());
        assert!(parse_positive_integer("abc", "test").is_err());
    }

    #[test]
    fn test_config_validation() {
        let mut config = Config::new_config();
        assert!(config.validate().is_ok());

        config.setup.name = "".to_string();
        assert!(config.validate().is_err());

        config.setup.name = "Valid Name".to_string();
        config.setup.days = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_folder_creation() {
        let folder = Folder::new("test_id", Some("parent_id"), "Test Folder");
        assert_eq!(folder.id, "test_id");
        assert_eq!(folder.parent_id, Some("parent_id".to_string()));
        assert_eq!(folder.name, "Test Folder");
    }
}
