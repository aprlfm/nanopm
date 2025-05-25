extern crate chrono;
extern crate fs_extra;

use std::{
    io,
    time::{Instant, SystemTime},
};

use chrono::{DateTime, offset::Utc};
use fs_extra::dir::get_dir_content;
use serde::{Deserialize, Serialize};

use crate::{
    Query,
    config::{Config, ConfigError, QueryInfo, QuerySettings, QueryType, SortType},
    num_to_char,
};

#[derive(Debug)]
pub enum QueryError {
    IoError(std::io::Error),
    FsExtraError(fs_extra::error::Error),
    ConfigError(ConfigError),
    InvalidQuery(String),
}

impl std::fmt::Display for QueryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QueryError::IoError(e) => write!(f, "IO error: {}", e),
            QueryError::FsExtraError(e) => write!(f, "Filesystem error: {}", e),
            QueryError::ConfigError(e) => write!(f, "Config error: {}", e),
            QueryError::InvalidQuery(msg) => write!(f, "Invalid query: {}", msg),
        }
    }
}

impl std::error::Error for QueryError {}

impl From<ConfigError> for QueryError {
    fn from(error: ConfigError) -> Self {
        QueryError::ConfigError(error)
    }
}

impl From<std::io::Error> for QueryError {
    fn from(error: std::io::Error) -> Self {
        QueryError::IoError(error)
    }
}

impl From<fs_extra::error::Error> for QueryError {
    fn from(error: fs_extra::error::Error) -> Self {
        QueryError::FsExtraError(error)
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum QueryResult {
    GeneralResult(GeneralResult),
    RootResult(RootResult),
    DayResult(DayResult),
    CamResult(CamResult),
    SoundResult(SoundResult),
    FolderResult(FolderResult),
}

impl QueryResult {
    fn get_result_string(self) -> String {
        match self {
            QueryResult::GeneralResult(r) => {
                format!(
                    "[General Query]\n{}",
                    toml::to_string(&r).expect("Could not serialize general query result!")
                )
            }
            QueryResult::RootResult(r) => {
                format!(
                    "[Root Query]\n{}",
                    toml::to_string(&r).expect("Could not serialize root query result!")
                )
            }
            QueryResult::DayResult(r) => {
                format!(
                    "[Day Query]\n{}",
                    toml::to_string(&r).expect("Could not serialize days query result!")
                )
            }
            QueryResult::CamResult(r) => {
                format!(
                    "[Camera Query]\n{}",
                    toml::to_string(&r).expect("Could not serialize cameras query result!")
                )
            }
            QueryResult::SoundResult(r) => {
                format!(
                    "[Sound Source Query]\n{}",
                    toml::to_string(&r).expect("Could not serialize sound sources query result!")
                )
            }
            QueryResult::FolderResult(r) => {
                format!(
                    "[Folder Query]\n{}",
                    toml::to_string(&r).expect("Could not serialize folder query result!")
                )
            }
        }
    }

    fn get_general_result(&self) -> &GeneralResult {
        match self {
            QueryResult::GeneralResult(r) => r,
            _ => panic!("Not a General Result!"),
        }
    }

    fn get_size_for_sorting(&self) -> u64 {
        match self {
            QueryResult::GeneralResult(r) => r.total_size_u64,
            QueryResult::RootResult(_) => 0, // Root results don't participate in size sorting
            QueryResult::DayResult(r) => r.total_size_u64,
            QueryResult::CamResult(r) => r.total_size_u64,
            QueryResult::SoundResult(r) => r.total_size_u64,
            QueryResult::FolderResult(r) => r.total_size_u64,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Eq, Ord, PartialEq, PartialOrd, Clone)]
pub struct GeneralResult {
    path: String,
    folder_name: String,
    file_count: usize,
    total_size: String,
    #[serde(skip_serializing)]
    total_size_u64: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    runtime_ms: Option<u64>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RootResult {
    project_name: String,
    file_count: usize,
    total_size: String,
    shoot_days: usize,
    camera_count: usize,
    sound_source_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    runtime_ms: Option<u64>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DayResult {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    path: Option<String>,
    day: String,
    file_count: usize,
    total_size: String,
    #[serde(skip_serializing)]
    total_size_u64: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    runtime_ms: Option<u64>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CamResult {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    path: Option<String>,
    camera: String,
    file_count: usize,
    total_size: String,
    #[serde(skip_serializing)]
    total_size_u64: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    runtime_ms: Option<u64>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SoundResult {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    path: Option<String>,
    sound_source: String,
    file_count: usize,
    total_size: String,
    #[serde(skip_serializing)]
    total_size_u64: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    runtime_ms: Option<u64>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FolderResult {
    path: String,
    file_count: usize,
    total_size: String,
    #[serde(skip_serializing)]
    total_size_u64: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    runtime_ms: Option<u64>,
}

pub fn query(query_info: QueryInfo) -> Result<(), QueryError> {
    let start_time = Instant::now();

    match query_info.query {
        Query::General(sort_type) => {
            query_general(
                sort_type,
                query_info.config,
                query_info.settings,
                start_time,
            )?;
        }
        Query::Partial(types, sort_type) => {
            query_partial(
                types,
                sort_type,
                query_info.config,
                query_info.settings,
                start_time,
            )?;
        }
        Query::Folder(folders, sort_type) => {
            query_folders(
                folders,
                sort_type,
                query_info.config,
                query_info.settings,
                start_time,
            )?;
        }
        Query::None => {
            return Err(QueryError::InvalidQuery(
                "No query type specified".to_string(),
            ));
        }
    }
    Ok(())
}

pub fn query_partial(
    types_to_query: Vec<QueryType>,
    sort_type: SortType,
    config: Config,
    settings: QuerySettings,
    start_time: Instant,
) -> Result<(), QueryError> {
    let mut query_results: Vec<QueryResult> = Vec::new();

    for query_type in types_to_query {
        let mut new_query_results: Vec<QueryResult> = match query_type {
            QueryType::Root => vec![query_root(&config, &settings)?],
            QueryType::Days | QueryType::Cams | QueryType::Sound => {
                query_iterable(&config, &settings, query_type)?
            }
        };
        query_results.append(&mut new_query_results);
    }

    apply_sorting(&mut query_results, &sort_type);

    write_query_results(
        query_results,
        settings,
        Query::Partial(Vec::new(), SortType::None),
        start_time,
    )?;
    Ok(())
}

pub fn query_general(
    sort_type: SortType,
    config: Config,
    settings: QuerySettings,
    start_time: Instant,
) -> Result<(), QueryError> {
    let folders: &Vec<String> = &config.general_query_params;
    let root_path = format!("./{}", config.setup.name);

    let all_files = get_dir_content(&root_path)?;

    let mut query_results = Vec::new();

    for folder in folders {
        let mut found_file = false;
        for file in &all_files.directories {
            let file_name_with_separator = format!("{}%", file);
            if file_name_with_separator.contains(&format!("{}%", folder)) {
                let file_data = get_dir_content(file)?;
                found_file = true;
                query_results.push(QueryResult::GeneralResult(GeneralResult {
                    path: file.replace("\\", "/"),
                    folder_name: folder.clone(),
                    file_count: file_data.files.len(),
                    total_size: to_shorthand(file_data.dir_size),
                    total_size_u64: file_data.dir_size,
                    runtime_ms: if settings.include_runtime {
                        Some(start_time.elapsed().as_millis() as u64)
                    } else {
                        None
                    },
                }));
            }
        }
        if !found_file && !settings.quiet {
            println!(
                "The non-existent folder \"{}\" was omitted from the query!",
                folder
            );
        }
    }

    apply_sorting(&mut query_results, &sort_type);

    query_results.insert(0, query_root(&config, &settings)?);

    write_query_results(
        query_results,
        settings,
        Query::General(sort_type),
        start_time,
    )?;
    Ok(())
}

pub fn query_iterable(
    config: &Config,
    settings: &QuerySettings,
    query_type: QueryType,
) -> Result<Vec<QueryResult>, QueryError> {
    let root_path = format!("./{}", config.setup.name);
    let all_files = get_dir_content(&root_path)?;

    let mut query_results = Vec::new();

    let (count, pattern) = match query_type {
        QueryType::Days => (config.setup.days, String::from("[Iter]_DAY[Iter]")),
        QueryType::Cams => (config.setup.cameras, String::from("[Iter]_[Char]_CAM")),
        QueryType::Sound => (
            config.setup.sound_sources,
            String::from("[Iter]_[Char]_REC"),
        ),
        QueryType::Root => {
            return Err(QueryError::InvalidQuery(
                "Root query should not be handled here".to_string(),
            ));
        }
    };

    if settings.unique_entries {
        for i in 1..=count {
            let file_to_query = pattern
                .replace("[Iter]", &format!("{:02}", i))
                .replace("[Char]", &num_to_char(i).to_string());

            let mut found_file = false;
            for file in &all_files.directories {
                let file_name_with_separator = format!("{}%", file);
                if file_name_with_separator.contains(&format!("{}%", file_to_query)) {
                    let file_data = get_dir_content(file)?;
                    found_file = true;

                    let result = create_query_result(
                        query_type.clone(),
                        i,
                        Some(file.replace("\\", "/")),
                        file_data.files.len(),
                        file_data.dir_size,
                        settings,
                    );
                    query_results.push(result);
                }
            }
            if !found_file && !settings.quiet {
                println!(
                    "The non-existent folder \"{}\" was omitted from the query!",
                    file_to_query
                );
            }
        }
    } else {
        for i in 1..=count {
            let file_to_query = pattern
                .replace("[Iter]", &format!("{:02}", i))
                .replace("[Char]", &num_to_char(i).to_string());

            let mut found_file = false;
            let mut file_count: usize = 0;
            let mut total_size: u64 = 0;

            for file in &all_files.directories {
                let file_name_with_separator = format!("{}%", file);
                if file_name_with_separator.contains(&format!("{}%", file_to_query)) {
                    found_file = true;
                    let file_data = get_dir_content(file)?;
                    file_count += file_data.files.len();
                    total_size += file_data.dir_size;
                }
            }

            if found_file {
                let result = create_query_result(
                    query_type.clone(),
                    i,
                    None,
                    file_count,
                    total_size,
                    settings,
                );
                query_results.push(result);
            } else if !settings.quiet {
                println!(
                    "The non-existent folder \"{}\" was omitted from the query!",
                    file_to_query
                );
            }
        }
    }

    Ok(query_results)
}

fn create_query_result(
    query_type: QueryType,
    index: usize,
    path: Option<String>,
    file_count: usize,
    total_size: u64,
    settings: &QuerySettings,
) -> QueryResult {
    match query_type {
        QueryType::Days => QueryResult::DayResult(DayResult {
            path,
            day: format!("Day {}", index),
            file_count,
            total_size: to_shorthand(total_size),
            total_size_u64: total_size,
            runtime_ms: if settings.include_runtime {
                Some(0)
            } else {
                None
            },
        }),
        QueryType::Cams => QueryResult::CamResult(CamResult {
            path,
            camera: format!("{} Cam ({})", num_to_char(index), index),
            file_count,
            total_size: to_shorthand(total_size),
            total_size_u64: total_size,
            runtime_ms: if settings.include_runtime {
                Some(0)
            } else {
                None
            },
        }),
        QueryType::Sound => QueryResult::SoundResult(SoundResult {
            path,
            sound_source: format!("{} Rec ({})", num_to_char(index), index),
            file_count,
            total_size: to_shorthand(total_size),
            total_size_u64: total_size,
            runtime_ms: if settings.include_runtime {
                Some(0)
            } else {
                None
            },
        }),
        QueryType::Root => panic!("Root should not be handled here"),
    }
}

pub fn query_root(config: &Config, settings: &QuerySettings) -> Result<QueryResult, QueryError> {
    let root_path = format!("./{}", config.setup.name);
    let all_files = get_dir_content(&root_path)?;

    Ok(QueryResult::RootResult(RootResult {
        project_name: config.setup.name.clone(),
        file_count: all_files.files.len(),
        total_size: to_shorthand(all_files.dir_size),
        shoot_days: config.setup.days,
        camera_count: config.setup.cameras,
        sound_source_count: config.setup.sound_sources,
        runtime_ms: if settings.include_runtime {
            Some(0)
        } else {
            None
        },
    }))
}

pub fn query_folders(
    folders: Vec<String>,
    sort_type: SortType,
    config: Config,
    settings: QuerySettings,
    start_time: Instant,
) -> Result<(), QueryError> {
    let root_path = format!("./{}", config.setup.name);
    let all_files = get_dir_content(&root_path)?;

    let mut query_results = Vec::new();

    for folder in folders {
        let mut found_file = false;
        for file in &all_files.directories {
            let file_name_with_separator = format!("{}%", file);
            if file_name_with_separator.contains(&format!("{}%", folder)) {
                let file_data = get_dir_content(file)?;
                found_file = true;
                query_results.push(QueryResult::FolderResult(FolderResult {
                    path: file.replace("\\", "/"),
                    file_count: file_data.files.len(),
                    total_size: to_shorthand(file_data.dir_size),
                    total_size_u64: file_data.dir_size,
                    runtime_ms: if settings.include_runtime {
                        Some(start_time.elapsed().as_millis() as u64)
                    } else {
                        None
                    },
                }));
            }
        }
        if !found_file && !settings.quiet {
            println!(
                "The non-existent folder \"{}\" was omitted from the query!",
                folder
            );
        }
    }

    apply_sorting(&mut query_results, &sort_type);

    write_query_results(
        query_results,
        settings,
        Query::Folder(Vec::new(), SortType::None),
        start_time,
    )?;
    Ok(())
}

fn apply_sorting(query_results: &mut Vec<QueryResult>, sort_type: &SortType) {
    match sort_type {
        SortType::BySize => {
            query_results.sort_by(|a, b| b.get_size_for_sorting().cmp(&a.get_size_for_sorting()));
        }
        SortType::ByDefaultOrder => {}
        SortType::None => {}
    }
}

pub fn write_query_results(
    query_results: Vec<QueryResult>,
    settings: QuerySettings,
    query_type: Query,
    start_time: Instant,
) -> Result<(), QueryError> {
    let mut full_text = String::new();
    for query_result in query_results {
        let text = query_result.get_result_string();
        full_text.push_str(&format!("{}\n", text));
    }

    let explanation_string = get_explanation_string(&query_type, &settings);
    let timestamp_string = get_timestamp_string(&settings);
    let runtime_string = if settings.include_runtime {
        format!(
            "Total Query Runtime: {}ms\n\n",
            start_time.elapsed().as_millis()
        )
    } else {
        String::new()
    };

    let export_path = get_export_path(&settings);

    println!("\n{}{}{}", explanation_string, runtime_string, full_text);

    if settings.write {
        write_to_file(
            &export_path,
            &timestamp_string,
            &explanation_string,
            &runtime_string,
            &full_text,
        )?;
    }

    Ok(())
}

fn get_explanation_string(query_type: &Query, settings: &QuerySettings) -> String {
    match query_type {
        Query::Partial(_, _) => {
            if settings.unique_entries {
                "unique_entries = true # Entries from different days will be displayed \
                 separately.\n\n"
            } else {
                "unique_entries = false # Entries from different days will be combined.\n\n"
            }
        }
        Query::General(sort_type) => match sort_type {
            SortType::ByDefaultOrder => "General Project Query - Sorted in Default Order\n\n",
            SortType::BySize => "General Project Query - Sorted by Size\n\n",
            SortType::None => "General Project Query\n\n",
        },
        Query::Folder(_, sort_type) => match sort_type {
            SortType::BySize => "Folder Query - Sorted by Size\n\n",
            SortType::ByDefaultOrder => "Folder Query - Default Order\n\n",
            SortType::None => "Folder Query\n\n",
        },
        Query::None => "",
    }
    .to_string()
}

fn get_timestamp_string(settings: &QuerySettings) -> String {
    if settings.record_timestamp {
        let system_time = SystemTime::now();
        let datetime: DateTime<Utc> = system_time.into();
        format!("{}\n", datetime.format("%d/%m/%Y %T"))
    } else {
        String::new()
    }
}

fn get_export_path(settings: &QuerySettings) -> String {
    match &settings.output_name {
        Some(path) => format!("{}.txt", path),
        None => {
            let system_time = SystemTime::now();
            let datetime: DateTime<Utc> = system_time.into();
            format!(
                "Query_{}.txt",
                datetime.format("%d.%m.%Y_%T").to_string().replace(":", ".")
            )
        }
    }
}

fn write_to_file(
    export_path: &str,
    timestamp_string: &str,
    explanation_string: &str,
    runtime_string: &str,
    full_text: &str,
) -> Result<(), QueryError> {
    if std::fs::exists(export_path)? {
        println!(
            "A file with the name {} already exists! Overwrite? (Y/N)",
            export_path
        );
        let mut response = String::new();
        io::stdin().read_line(&mut response)?;
        if response.trim().to_lowercase() != "y" && response.trim().to_lowercase() != "yes" {
            println!("Did not overwrite existing file.");
            return Ok(());
        }
    }

    let content = format!(
        "{}{}{}{}",
        timestamp_string, explanation_string, runtime_string, full_text
    );
    std::fs::write(export_path, content)?;
    println!("Query result written to: {}", export_path);
    Ok(())
}

pub fn to_shorthand(bytes: u64) -> String {
    if bytes == 0 {
        return "0B".to_string();
    }

    let mut current_number: f64 = bytes as f64;
    let mut expo_10: u32 = 0;

    while current_number >= 1024f64 {
        current_number /= 1024f64;
        expo_10 += 3;
    }

    let (decimal_points, prefix) = match expo_10 {
        0 => (0, ""),
        3 => (0, "K"),
        6 => (1, "M"),
        9 => (1, "G"),
        12 => (1, "T"),
        15 => (1, "P"),
        18 => (1, "E"),
        21 => (1, "Z"),
        24 => (1, "Y"),
        _ => return format!("{}B", bytes),
    };

    let decimal_points = if current_number >= 100.0 {
        0
    } else {
        decimal_points
    };

    let ib = (current_number * 10f64.powi(decimal_points as i32)).round()
        / 10f64.powi(decimal_points as i32);

    let b = bytes as f64 / 1000_f64.powi((expo_10 / 3) as i32);

    if expo_10 == 0 {
        format!("{:.0}B", bytes)
    } else {
        match decimal_points {
            0 => {
                let ib_rounded = ib.round() as u64;
                let b_rounded = b.round() as u64;
                format!("{}{}iB ({}{}B)", ib_rounded, prefix, b_rounded, prefix)
            }
            1 => format!("{:.1}{}iB ({:.1}{}B)", ib, prefix, b, prefix),
            _ => format!("{:.2}{}iB ({:.2}{}B)", ib, prefix, b, prefix),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_shorthand() {
        assert_eq!(to_shorthand(0), "0B");
        assert_eq!(to_shorthand(512), "512B");
        assert_eq!(to_shorthand(1536), "2KiB (2KB)");
        assert_eq!(to_shorthand(1048576), "1.0MiB (1.0MB)");
        assert_eq!(to_shorthand(1073741824), "1.0GiB (1.1GB)");
    }

    #[test]
    fn test_get_explanation_string() {
        let query = Query::General(SortType::BySize);
        let settings = QuerySettings::default();
        let explanation = get_explanation_string(&query, &settings);
        assert!(explanation.contains("General Project Query - Sorted by Size"));
    }

    #[test]
    fn test_get_export_path() {
        let mut settings = QuerySettings::default();
        settings.output_name = Some("test_output".to_string());
        assert_eq!(get_export_path(&settings), "test_output.txt");

        settings.output_name = None;
        let path = get_export_path(&settings);
        assert!(path.starts_with("Query_"));
        assert!(path.ends_with(".txt"));
    }

    #[test]
    fn test_create_query_result() {
        let settings = QuerySettings::default();
        let result = create_query_result(QueryType::Days, 1, None, 10, 1024, &settings);

        match result {
            QueryResult::DayResult(day_result) => {
                assert_eq!(day_result.day, "Day 1");
                assert_eq!(day_result.file_count, 10);
                assert!(day_result.total_size.contains("1"));
            }
            _ => panic!("Expected DayResult"),
        }
    }

    #[test]
    fn test_query_error_display() {
        let query_error = QueryError::InvalidQuery("Bad query".to_string());
        assert!(format!("{}", query_error).contains("Invalid query"));

        let io_error = QueryError::IoError(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "Access denied",
        ));
        assert!(format!("{}", io_error).contains("IO error"));
    }

    #[test]
    fn test_size_sorting() {
        let mut query_results = vec![
            QueryResult::FolderResult(FolderResult {
                path: "test1".to_string(),
                file_count: 5,
                total_size: "1MB".to_string(),
                total_size_u64: 1048576,
                runtime_ms: None,
            }),
            QueryResult::FolderResult(FolderResult {
                path: "test2".to_string(),
                file_count: 3,
                total_size: "2MB".to_string(),
                total_size_u64: 2097152,
                runtime_ms: None,
            }),
        ];

        apply_sorting(&mut query_results, &SortType::BySize);

        if let QueryResult::FolderResult(first) = &query_results[0] {
            assert_eq!(first.total_size_u64, 2097152);
        } else {
            panic!("Expected FolderResult");
        }
    }
}
