extern crate fs_extra;
extern crate chrono;

use chrono::offset::Utc;
use chrono::DateTime;
use core::panic;
use std::io;
use std::time::SystemTime;
use fs_extra::dir::get_dir_content;
use crate::{Query, num_to_char};
use crate::config::{Config, QueryInfo, QuerySettings, QueryType, SortType};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub enum QueryResult{
    GeneralResult(GeneralResult),
    RootResult(RootResult),
    DayResult(DayResult),
    CamResult(CamResult),
    SoundResult(SoundResult),
    FolderResult(FolderResult),
}

impl QueryResult{
    fn get_result_string(self) -> String {
        match self {
            QueryResult::GeneralResult(r) => {
                format!("[General Query]\n{}",toml::to_string(&r).expect("Could not serialize general query result!"))
            },
            QueryResult::RootResult(r) => {
                format!("[Root Query]\n{}",toml::to_string(&r).expect("Could not serialize root query result!"))
            },
            QueryResult::DayResult(r) => {
                format!("[Day Query]\n{}",toml::to_string(&r).expect("Could not serialize days query result!"))
        },
            QueryResult::CamResult(r) => {
                format!("[Camera Query]\n{}",toml::to_string(&r).expect("Could not serialize cameras query result!"))
        },
            QueryResult::SoundResult(r) => {
                format!("[Sound Source Query]\n{}",toml::to_string(&r).expect("Could not serialize sound sources query result!"))
        },
            QueryResult::FolderResult(r) => {
                format!("[Folder Query]\n{}",toml::to_string(&r).expect("Could not serialize folder query result!"))
        },
        }
    }

    fn get_general_result(&self) -> &GeneralResult {
        match self {
            QueryResult::GeneralResult(r) => {
                r
            },
            _ => {panic!("Not a General Result!")}
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Eq, Ord, PartialEq, PartialOrd)]
pub struct GeneralResult{
    path: String,
    folder_name : String,
    file_count : usize,
    total_size : String,
    #[serde(skip_serializing)]
    total_size_u64 : u64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RootResult{
    project_name : String,
    file_count : usize,
    total_size : String,
    shoot_days: usize,
    camera_count: usize,
    sound_source_count: usize,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DayResult{
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    path: Option<String>,
    day : String,
    file_count : usize,
    total_size : String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CamResult{
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    path: Option<String>,
    camera : String,
    file_count : usize,
    total_size : String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SoundResult{
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    path: Option<String>,
    sound_source : String,
    file_count : usize,
    total_size : String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FolderResult{
    path : String,
    file_count : usize,
    total_size : String,
}

pub fn query(query: QueryInfo) {
    match query.query {
        Query::General(sort_type) => {query_general(sort_type, query.config, query.settings);},
        Query::Partial(types) => {query_partial(types, query.config, query.settings);},
        Query::Folder(folders) => {query_folders(folders, query.config, query.settings);},
        Query::None => {}, // Unreachable
    }
}

pub fn query_partial(types_to_query: Vec<QueryType>, config: Config, settings: QuerySettings) {
    let mut query_results: Vec<QueryResult> = Vec::new();
    for query in types_to_query {
        let mut new_query_results: Vec<QueryResult> = match query {
            QueryType::Root => {vec![query_root(&config)]},
            QueryType::Days | QueryType::Cams | QueryType::Sound => {query_iterable(&config, &settings, query)},
        };
        query_results.append(&mut new_query_results);
    }
    write_query_results(query_results, settings, Query::Partial(Vec::new()));
}

pub fn query_general(sort_type: SortType, config: Config, settings: QuerySettings) {
    let folders: &Vec<String> = &config.general_query_params;
    let root_path = &format!("./{}",config.setup.name);
    // dbg!(&config);
    let mut all_files = get_dir_content(root_path).expect("Could not get directory content!");
    let mut query_results = Vec::new();
    for folder in folders {
        let file_to_query = folder;
        let mut found_file = false;
        for file in &mut all_files.directories {
            let mut file_name_percent = file.clone();
            file_name_percent.push('%');
            // Push % to file name to see if the string is at the end of the directory
            // dbg!(&format!("{}%", &folder));
            // dbg!(&file_name_percent);
            match &file_name_percent.find(&format!("{}%", &file_to_query)) {
                Some(_) => {
                    let file_data = get_dir_content(&file).expect("Could not get directory content!");
                    found_file = true;
                    query_results.push(QueryResult::GeneralResult(
                        GeneralResult{
                            path: file.replace("\\", "/"),
                            folder_name: folder.clone(),
                            file_count: file_data.files.len(), 
                            total_size: to_shorthand(file_data.dir_size),
                            total_size_u64: file_data.dir_size,
                        }
                    ));
                }
                None => {}
            }
        }
        if !found_file{
            if !settings.quiet {
                println!("The non-existent folder \"{}\" was omitted from the query!", file_to_query);
            }
        }
    }
    match sort_type {
        SortType::BySize => {
            query_results.sort_by(|a, b| 
                b.get_general_result().total_size_u64
                .cmp(&a.get_general_result().total_size_u64));
        },
        SortType::ByDefaultOrder => {
            // No sorting required
        },
        SortType::None => {
            // Inaccessible
        },
    }
    query_results.insert(0, query_root(&config));
    write_query_results(query_results, settings, Query::General(sort_type));
}

pub fn query_iterable(config: &Config, settings: &QuerySettings, query: QueryType) -> Vec<QueryResult> {
    let root_path = &format!("./{}",config.setup.name);
    let mut all_files = get_dir_content(root_path).expect("Could not get directory content!");
    let mut query_results = Vec::new();
    let query_specific_params: (usize, String);
        match query {
            QueryType::Days => {
                query_specific_params = (config.setup.days, String::from("[Iter]_DAY[Iter]"));
            },
            QueryType::Cams => {
                query_specific_params = (config.setup.cameras, String::from("[Iter]_[Char]_CAM"));
            },
            QueryType::Sound => {
                query_specific_params = (config.setup.sound_sources, String::from("[Iter]_[Char]_REC"));
            },
            QueryType::Root => {panic!("This should be inaccessible!")}, // Inaccessible
        }
    if settings.unique_entries {
        for iter in 1..query_specific_params.0 + 1 {
            let file_to_query = &query_specific_params.1
            .replace("[Iter]", &format!("{:0>2}", iter))
            .replace("[Char]", &num_to_char(iter).to_string());
            let mut found_file = false;
            for file in &mut all_files.directories {
                let mut file_name_percent = file.clone();
                file_name_percent.push('%');
                // Push % to file name to see if the string is at the end of the directory
                match &file_name_percent.find(&format!("{}%", &file_to_query)) {
                    Some(_) => {
                        let file_data = get_dir_content(&file).expect("Could not get directory content!");
                        found_file = true;
                        query_results.push(match query {
                            QueryType::Days => {
                                QueryResult::DayResult(
                                    DayResult{
                                        path: Some(file.replace("\\", "/")),
                                        day: String::from("Day ") + &iter.to_string(),
                                        file_count: file_data.files.len(), 
                                        total_size: to_shorthand(file_data.dir_size),
                                    }
                            )},
                            QueryType::Cams => {
                                QueryResult::CamResult(
                                    CamResult{
                                        path: Some(file.replace("\\", "/")),
                                        camera: format!("{} Cam ({})",num_to_char(iter),iter.to_string()),
                                        file_count: file_data.files.len(), 
                                        total_size: to_shorthand(file_data.dir_size),
                                    }
                            )},
                            QueryType::Sound => {
                                QueryResult::SoundResult(
                                    SoundResult{
                                        path: Some(file.replace("\\", "/")),
                                        sound_source: format!("{} Rec ({})",num_to_char(iter),iter.to_string()),
                                        file_count: file_data.files.len(), 
                                        total_size: to_shorthand(file_data.dir_size),
                                    }
                            )},
                            QueryType::Root => {panic!("This should be inaccessible!")} // Inaccessible
                        })
                    }
                    None => {}
                }
            }
            if !found_file{
                if !settings.quiet {
                    println!("The non-existent folder \"{}\" was omitted from the query!", file_to_query);
                }
            }
        }
    } else {
        for iter in 1..query_specific_params.0 + 1 {
            let file_to_query = &query_specific_params.1
            .replace("[Iter]", &format!("{:0>2}", iter))
            .replace("[Char]", &num_to_char(iter).to_string());
            let mut found_file = false;
            let mut file_count: usize = 0;
            let mut total_size: u64 = 0;
            for file in &mut all_files.directories {
                let mut file_name_percent = file.clone();
                file_name_percent.push('%');
                // Push % to file name to see if the string is at the end of the directory
                match &file_name_percent.find(&format!("{}%", &file_to_query)) {
                    Some(_) => {
                        found_file = true;
                        let file_data = get_dir_content(&file).expect("Could not get directory content!");
                        file_count += file_data.files.len();
                        total_size += file_data.dir_size;
                    }
                    None => {}
                }
            }
            if found_file {
                query_results.push(match query {
                    QueryType::Days => {
                        QueryResult::DayResult(
                            DayResult{
                                path: None,
                                day: String::from("Day ") + &iter.to_string(),
                                file_count, 
                                total_size: to_shorthand(total_size),
                            }
                    )},
                    QueryType::Cams => {
                        QueryResult::CamResult(
                            CamResult{
                                path: None,
                                camera: format!("{} Cam ({})",num_to_char(iter),iter.to_string()),
                                file_count, 
                                total_size: to_shorthand(total_size),
                            }
                    )},
                    QueryType::Sound => {
                        QueryResult::SoundResult(
                            SoundResult{
                                path: None,
                                sound_source: format!("{} Rec ({})",num_to_char(iter),iter.to_string()),
                                file_count, 
                                total_size: to_shorthand(total_size),
                            }
                    )},
                    QueryType::Root => {panic!("This should be inaccessible!")} // Inaccessible
                });
            } else {
                if !settings.quiet {
                    println!("The non-existent folder \"{}\" was omitted from the query!", file_to_query);
                }
            }
        }
    };
    query_results
}


pub fn query_root(config: &Config) -> QueryResult{
    let root_path = &format!("./{}",config.setup.name);
    let all_files = get_dir_content(root_path).expect(&format!("Could not find the base directory at \"{}\"!", root_path));
    QueryResult::RootResult( 
        RootResult{
            project_name: config.setup.name.to_string(),
            file_count: all_files.files.len(), 
            total_size: to_shorthand(all_files.dir_size),
            shoot_days: config.setup.days,
            camera_count: config.setup.cameras,
            sound_source_count: config.setup.sound_sources,
        }
    )
}

// if multiple folders share the same name, each will be printed individually.
pub fn query_folders(folders : Vec<String>, config: Config, settings: QuerySettings) {
    let root_path = &format!("./{}",config.setup.name);
    let mut all_files = get_dir_content(root_path).expect("Could not get directory content!");
    let mut query_results = Vec::new();
    for folder in folders {
        let file_to_query = folder;
            let mut found_file = false;
        for file in &mut all_files.directories {
            let mut file_name_percent = file.clone();
            file_name_percent.push('%');
            // Push % to file name to see if the string is at the end of the directory
            // dbg!(&format!("{}%", &folder));
            // dbg!(&file_name_percent);
            match &file_name_percent.find(&format!("{}%", &file_to_query)) {
                Some(_) => {
                    let file_data = get_dir_content(&file).expect("Could not get directory content!");
                    found_file = true;
                    query_results.push(QueryResult::FolderResult(
                        FolderResult{
                            path: file.replace("\\", "/"),
                            file_count: file_data.files.len(), 
                            total_size: to_shorthand(file_data.dir_size),
                        }
                    ));
                }
                None => {}
            }
        }
        if !found_file{
            if !settings.quiet {
                println!("The non-existent folder \"{}\" was omitted from the query!", file_to_query);
            }
        }
    }
    write_query_results(query_results, settings, Query::Folder(Vec::new()));
}

pub fn write_query_results(query_results: Vec<QueryResult>, settings: QuerySettings, query_type: Query) {
    let mut full_text = String::from("");
    for query_result in query_results {
        let text = query_result.get_result_string();
        full_text.push_str(&format!("{}\n",text));
    }
    let timestamp: String;
    let explanation_string: &str;
    let system_time = SystemTime::now();
    let datetime: DateTime<Utc> = system_time.into();
    if query_type == Query::Partial(Vec::new()) {
        explanation_string = if settings.unique_entries {
            "unique_entries = true # Entries from different days will be displayed separately.\n\n"
        } else {
            "unique_entries = false # Entries from different days will be combined.\n\n"
        };
    } else if query_type == Query::General(SortType::any()) {
        explanation_string = match query_type.get_sort_type() {
            SortType::ByDefaultOrder => {
                "General Project Query - Sorted in Default Order\n\n"
            },
            SortType::BySize => {
                "General Project Query - Sorted by Size\n\n"
            }
            SortType::None => {
                panic!("No sort type specified in general query even after passed into actual query!")
            }
        }
    } else {
        explanation_string = "";
    }
    if settings.record_timestamp{
        timestamp = datetime.format("%d/%m/%Y %T").to_string() + 
        "\n" + 
        if explanation_string == "" {"\n"} else {""};
    } else {
        timestamp = String::from("");
    }
    let export_path = match settings.output_name{
        Some(path) => path + ".txt",
        None => String::from(format!("Query_{}.txt", datetime.format("%d.%m.%Y_%T").to_string().replace(":", "."))),
    };

    println!("\n{}{}", explanation_string, full_text);

    if settings.write {
        if !std::fs::exists(&export_path).expect("Can't check existence of file does_not_exist.txt"){
            std::fs::write(&export_path, format!("{}{}{}", timestamp, explanation_string, full_text)).expect("Failed to write query result!");
            println!("Wrote query to file at {}", &export_path);
        } else {
            println!("A file with the name {} already exists! Overwrite? (Y/N)", &export_path);
            let mut response = String::new();
            io::stdin().read_line(&mut response).expect("Failed to read line");
            if response.trim() == "Y" || response.trim() == "y" {
                std::fs::write(&export_path, format!("{}{}{}", timestamp, explanation_string, full_text)).expect("Failed to write query result!");
                println!("Overwrote query to file at {}", export_path);
            } else {
                println!("Did not overwrite existing file.");
            }
        }
    }
}

pub fn to_shorthand(bytes: u64) -> String {
    let mut current_number: f64 = bytes as f64;
    let mut expo_10: u32 = 0;
    while current_number > 10240f64 || (expo_10 == 0 && current_number > 1024f64) {
        current_number /= 1024f64;
        expo_10 += 3;
    };
    let mut decimal_points: u32;
    let prefix: &str = match expo_10 {
        0 => {decimal_points = 0; ""},
        3 => {decimal_points = 0; "K"},
        6 => {decimal_points = 1; "M"},
        9 => {decimal_points = 1; "G"},
        12 => {decimal_points = 1; "T"},
        15 => {decimal_points = 1; "P"},
        18 => {decimal_points = 1; "E"},
        21 => {decimal_points = 1; "Z"},
        24 => {decimal_points = 1; "Y"},
        _ => panic!("Not a valid exponent!"),
    };
    if current_number >= 100.0 {
        decimal_points = 0;
    }
    let ib = ((current_number) * 10f64.powf(decimal_points as f64)).round() / 10f64.powf(decimal_points as f64) as f64;
    let b:f64 = ((bytes * 10u64.pow(decimal_points)) / (10u64.pow(expo_10))) as f64 / 10u64.pow(decimal_points) as f64;
    if expo_10 == 0 {
        String::from(format!("{b:.0}B"))
    } else {
        String::from(format!("{ib}{prefix}iB ({b}{prefix}B)"))
    }
}