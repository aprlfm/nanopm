extern crate fs_extra;
extern crate chrono;

use chrono::offset::Utc;
use chrono::DateTime;
use std::time::SystemTime;
use fs_extra::dir::get_dir_content;
use crate::{Query, num_to_char};
use crate::config::{Config, QueryInfo, QuerySettings, QueryType};
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
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GeneralResult{

}

#[derive(Debug, Deserialize, Serialize)]
pub struct RootResult{
    project_name : String,
    file_count : usize,
    total_size : u64,
    shoot_days: usize,
    camera_count: usize,
    sound_source_count: usize,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DayResult{
    day : String,
    file_count : usize,
    total_size : u64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CamResult{
    camera : String,
    file_count : usize,
    total_size : u64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SoundResult{
    sound_recorder : String,
    file_count : usize,
    total_size : u64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FolderResult{
    path : String,
    file_count : usize,
    total_size : u64,
}

pub fn query(query: QueryInfo, config: Config) {
    match query.query {
        Query::General => {query_general(config, query.settings);},
        Query::Partial(types) => {query_partial(types, config, query.settings);},
        Query::Folder(folders) => {query_folders(folders, config, query.settings);},
        Query::None => {}, // Unreachable
    }
}

pub fn query_partial(types_to_query: Vec<QueryType>, config: Config, settings: QuerySettings){
    let mut query_results: Vec<QueryResult> = Vec::new();
    for query in types_to_query {
        let mut new_query_results: Vec<QueryResult> = match query {
            QueryType::Root => {query_root(&config)},
            QueryType::Days => {query_by_day(&config)},
            QueryType::Cams => {query_by_cam(&config)},
            QueryType::Sound => {query_by_sound_recorder(&config)},
        };
        query_results.append(&mut new_query_results);
    };
    if settings.write {
        write_query_results(query_results, settings);
    }
}

pub fn query_general(config: Config, settings: QuerySettings) {

}

pub fn query_root(config: &Config) -> Vec<QueryResult>{
    let root_path = &format!("./{}",config.setup.name);
    let all_files = get_dir_content(root_path).expect("Could not get directory content!");
    vec![QueryResult::RootResult( 
        RootResult{
            project_name: config.setup.name.to_string(),
            file_count: all_files.files.len(), 
            total_size: all_files.dir_size,
            shoot_days: config.setup.days,
            camera_count: config.setup.cameras,
            sound_source_count: config.setup.sound_sources,
        }
    )]
}

pub fn query_by_day(config: &Config) -> Vec<QueryResult> {
    let root_path = &format!("./{}",config.setup.name);
    let mut all_files = get_dir_content(root_path).expect("Could not get directory content!");
    let mut query_results = Vec::new();
    for day_iter in 1..config.setup.cameras + 1 {
        let mut file_count: usize = 0;
        let mut total_size: u64 = 0;
        for file in &mut all_files.directories {
            let padded_number = format!("{:0>2}", day_iter);
            let mut file_name_percent = file.clone();
            file_name_percent.push('%');
            // Push % to file name to see if the string is at the end of the directory
            match &file_name_percent.find(&format!("{}_DAY{}%", padded_number, padded_number)) {
                Some(_) => {
                    let file_data = get_dir_content(file).expect("Could not get directory content!");
                    file_count += file_data.files.len();
                    total_size += file_data.dir_size;
                }
                None => {}
            }
        }
        query_results.push(QueryResult::DayResult(
            DayResult{
                day: String::from("Day") + &day_iter.to_string(),
                file_count, total_size,
            }
        ));
    }
    query_results
}

pub fn query_by_cam(config: &Config) -> Vec<QueryResult> {
    let root_path = &format!("./{}",config.setup.name);
    let mut all_files = get_dir_content(root_path).expect("Could not get directory content!");
    let mut query_results = Vec::new();
    for camera_iter in 1..config.setup.cameras + 1 {
        let mut file_count: usize = 0;
        let mut total_size: u64 = 0;
        for file in &mut all_files.directories {
            let padded_number = format!("{:0>2}", camera_iter);
            let mut file_name_percent = file.clone();
            file_name_percent.push('%');
            // Push % to file name to see if the string is at the end of the directory
            match &file_name_percent.find(&format!("{x}_{y}_CAM%", x = padded_number, y = num_to_char(camera_iter).to_string())) {
                Some(_) => {
                    let file_data = get_dir_content(file).expect("Could not get directory content!");
                    file_count += file_data.files.len();
                    total_size += file_data.dir_size;
                }
                None => {}
            }
        }
        query_results.push(QueryResult::CamResult(
            CamResult{
                camera: format!("{} CAM ({})",num_to_char(camera_iter),camera_iter.to_string()),
                file_count, total_size,
            }
        ));
    }
    query_results
}

pub fn query_by_sound_recorder(config: &Config) -> Vec<QueryResult> {
    let root_path = &format!("./{}",config.setup.name);
    let mut all_files = get_dir_content(root_path).expect("Could not get directory content!");
    let mut query_results = Vec::new();
    for sound_iter in 1..config.setup.sound_sources + 1 {
        let mut file_count: usize = 0;
        let mut total_size: u64 = 0;
        for file in &mut all_files.directories {
            let padded_number = format!("{:0>2}", sound_iter);
            let mut file_name_percent = file.clone();
            file_name_percent.push('%');
            // Push % to file name to see if the string is at the end of the directory
            match &file_name_percent.find(&format!("{x}_{y}_REC%", x = padded_number, y = num_to_char(sound_iter).to_string())) {
                Some(_) => {
                    let file_data = get_dir_content(file).expect("Could not get directory content!");
                    file_count += file_data.files.len();
                    total_size += file_data.dir_size;
                }
                None => {}
            }
        }
        query_results.push(QueryResult::SoundResult(
            SoundResult{
                sound_recorder: format!("{} REC ({})",num_to_char(sound_iter),sound_iter.to_string()),
                file_count, total_size,
            }
        ));
    }
    query_results
}

// if multiple folders share the same name, each will be printed individually.
pub fn query_folders(folders : Vec<String>, config: Config, settings: QuerySettings) {
    let root_path = &format!("./{}",config.setup.name);
    let mut all_files = get_dir_content(root_path).expect("Could not get directory content!");
    let mut query_results = Vec::new();
    for folder in folders {
        for file in &mut all_files.directories {
            let mut file_name_percent = file.clone();
            file_name_percent.push('%');
            // Push % to file name to see if the string is at the end of the directory
            // dbg!(&format!("{}%", &folder));
            // dbg!(&file_name_percent);
            match &file_name_percent.find(&format!("{}%", &folder)) {
                Some(_) => {
                    let file_data = get_dir_content(&file).expect("Could not get directory content!");
                    query_results.push(QueryResult::FolderResult(
                        FolderResult{
                            path: file.replace("\\", "/"),
                            file_count: file_data.files.len(), 
                            total_size: file_data.dir_size,
                        }
                    ));
                }
                None => {}
            }
        }
    }
    write_query_results(query_results, settings);
}

pub fn write_query_results(query_results: Vec<QueryResult>, settings: QuerySettings) {
    let mut full_text = String::from("");
    for query_result in query_results {
        let text = query_result.get_result_string();
        full_text.push_str(&format!("{}\n",text));
    }
    let system_time = SystemTime::now();
    let datetime: DateTime<Utc> = system_time.into();
    if settings.record_timestamp{
        let prepend = datetime.format("%d/%m/%Y %T").to_string() + "\n\n";
        full_text.insert_str(0,&prepend);
    }
    let export_path = match settings.output_name{
        Some(path) => path + ".txt",
        None => String::from(format!("Query_{}.txt", datetime.format("%d/%m/%Y %T"))),
    };
    std::fs::write(export_path, full_text).expect("Failed to write query result!");
}