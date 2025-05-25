




#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;
    use tempfile::TempDir;

    use crate::util::config::*;
    use crate::util::init::*;
    use crate::util::query::*;
    use crate::util::util::*;


    struct TestProject {
        temp_dir: TempDir,
        config: Config,
    }

    impl TestProject {
        fn new(name: &str) -> Self {
            let temp_dir = TempDir::new().expect("Failed to create temp directory");
            let mut config = Config::new_config();
            config.setup.name = name.to_string();


            std::env::set_current_dir(temp_dir.path()).expect("Failed to change directory");

            TestProject { temp_dir, config }
        }

        fn create_project_structure(&self) -> Result<(), Box<dyn std::error::Error>> {

            fs::create_dir(&self.config.setup.name)?;


            let project_path = Path::new(&self.config.setup.name);
            fs::create_dir_all(project_path.join("01_DOCUMENTATION"))?;
            fs::create_dir_all(project_path.join("02_RUSHES/01_DAY01/01_VIDEO/01_A_CAM"))?;
            fs::create_dir_all(project_path.join("02_RUSHES/01_DAY01/02_AUDIO/01_A_REC"))?;
            fs::create_dir_all(project_path.join("03_EXTERNAL/01_GRAPHICS"))?;


            fs::write(project_path.join("01_DOCUMENTATION/script.pdf"), b"test file")?;
            fs::write(project_path.join("02_RUSHES/01_DAY01/01_VIDEO/01_A_CAM/clip001.mov"), b"video data")?;
            fs::write(project_path.join("02_RUSHES/01_DAY01/02_AUDIO/01_A_REC/audio001.wav"), b"audio data")?;

            Ok(())
        }

        fn path(&self) -> &Path {
            self.temp_dir.path()
        }
    }


    mod config_tests {
        use super::*;

        #[test]
        fn test_config_creation() {
            let config = Config::new_config();
            assert_eq!(config.version, "v2");
            assert_eq!(config.setup.name, "Untitled_Project");
            assert_eq!(config.setup.days, 2);
            assert_eq!(config.setup.cameras, 2);
            assert_eq!(config.setup.sound_sources, 1);
            assert!(!config.general_query_params.is_empty());
        }

        #[test]
        fn test_config_validation() {
            let mut config = Config::new_config();


            assert!(config.validate().is_ok());


            config.setup.name = "".to_string();
            assert!(config.validate().is_err());


            config.setup.name = "Valid".to_string();
            config.setup.days = 0;
            assert!(config.validate().is_err());


            config.setup.days = 1;
            config.setup.cameras = 0;
            assert!(config.validate().is_err());


            config.setup.cameras = 1;
            config.setup.sound_sources = 0;
            assert!(config.validate().is_err());
        }

        #[test]
        fn test_folder_structure() {
            let structure = FileStructure::get_default_structure();
            assert!(!structure.folders_list.is_empty());


            let folder_names: Vec<&String> = structure.folders_list.iter().map(|f| &f.name).collect();
            assert!(folder_names.contains(&&"01_DOCUMENTATION".to_string()));
            assert!(folder_names.contains(&&"02_RUSHES".to_string()));
            assert!(folder_names.contains(&&"%days".to_string()));
        }

        #[test]
        fn test_folder_creation() {
            let folder = Folder::new("test_id", Some("parent_id"), "Test Folder");
            assert_eq!(folder.id, "test_id");
            assert_eq!(folder.parent_id, Some("parent_id".to_string()));
            assert_eq!(folder.name, "Test Folder");

            let root_folder = Folder::new("root", None, "Root Folder");
            assert_eq!(root_folder.parent_id, None);
        }

        #[test]
        fn test_config_serialization() {
            let config = Config::new_config();
            let serialized = toml::to_string(&config).expect("Failed to serialize config");
            assert!(serialized.contains("version"));
            assert!(serialized.contains("name"));
            assert!(serialized.contains("days"));
        }
    }


    mod init_tests {
        use super::*;

        #[test]
        fn test_project_setup_creation() {
            let setup = ProjectSetup::new();
            assert_eq!(setup.name, "Untitled_Project");
            assert_eq!(setup.days, 2);
            assert_eq!(setup.cameras, 2);
            assert_eq!(setup.sound_sources, 1);
            assert!(!setup.clean_project);
            assert!(setup.deadname.is_none());
        }

        #[test]
        fn test_project_setup_validation() {
            let mut setup = ProjectSetup::new();
            assert!(setup.validate().is_ok());

            setup.name = "".to_string();
            assert!(setup.validate().is_err());

            setup.name = "Valid".to_string();
            setup.days = 0;
            assert!(setup.validate().is_err());
        }

        #[test]
        fn test_operation_types() {
            assert_eq!(OperationType::New, OperationType::New);
            assert_ne!(OperationType::New, OperationType::Update);
            assert_ne!(OperationType::Update, OperationType::Query);
        }

        #[test]
        fn test_required_type_functions() {
            assert_eq!(get_required_type_init(InitParams::Days, true), "a positive integer");
            assert_eq!(get_required_type_init(InitParams::Days, false), "usize");
            assert_eq!(get_required_type_init(InitParams::ProjName, true), "a String");

            assert_eq!(get_required_type_query(QueryParams::Folder, true), "a String");
            assert_eq!(get_required_type_query(QueryParams::OutputDir, false), "String");
        }
    }


    mod query_tests {
        use super::*;

        #[test]
        fn test_query_info_creation() {
            let query_info = QueryInfo::new_query_info();
            assert!(matches!(query_info.query, Query::None));
            assert!(!query_info.settings.write);
            assert!(!query_info.settings.record_timestamp);
            assert!(!query_info.settings.unique_entries);
            assert!(!query_info.settings.quiet);
            assert!(!query_info.settings.include_runtime);
        }

        #[test]
        fn test_query_settings() {
            let mut settings = QuerySettings::default();
            assert!(!settings.write);
            assert!(settings.output_name.is_none());

            settings.write = true;
            settings.output_name = Some("test_output".to_string());
            settings.record_timestamp = true;
            settings.unique_entries = true;
            settings.quiet = true;
            settings.include_runtime = true;

            assert!(settings.write);
            assert_eq!(settings.output_name, Some("test_output".to_string()));
            assert!(settings.record_timestamp);
            assert!(settings.unique_entries);
            assert!(settings.quiet);
            assert!(settings.include_runtime);
        }

        #[test]
        fn test_sort_types() {
            assert_eq!(SortType::default_sort_type(), SortType::ByDefaultOrder);
            assert_ne!(SortType::BySize, SortType::ByDefaultOrder);
        }

        #[test]
        fn test_query_types() {
            let query = Query::General(SortType::BySize);
            assert_eq!(*query.get_sort_type(), SortType::BySize);

            let partial_query = Query::Partial(vec![QueryType::Root], SortType::ByDefaultOrder);
            assert_eq!(*partial_query.get_sort_type(), SortType::ByDefaultOrder);
        }

        #[test]
        fn test_to_shorthand() {
            assert_eq!(to_shorthand(0), "0B");
            assert_eq!(to_shorthand(1024), "1KiB (1KB)");
            assert_eq!(to_shorthand(1048576), "1.0MiB (1.0MB)");
        }

        #[test]
        fn test_get_default_general_query() {
            let query_params = Query::get_default_general_query();
            assert!(!query_params.is_empty());
            assert!(query_params.contains(&"01_DOCUMENTATION".to_string()));
            assert!(query_params.contains(&"01_VIDEO".to_string()));
            assert!(query_params.contains(&"05_FINALS".to_string()));
        }
    }


    mod util_tests {
        use super::*;

        #[test]
        fn test_get_version() {
            assert_eq!(get_version(), "v2");
        }

        #[test]
        fn test_format_duration() {
            assert_eq!(format_duration(0), "0ms");
            assert_eq!(format_duration(500), "500ms");
            assert_eq!(format_duration(1000), "1.00s");
            assert_eq!(format_duration(1500), "1.50s");
            assert_eq!(format_duration(60000), "1m 0s");
            assert_eq!(format_duration(65000), "1m 5s");
            assert_eq!(format_duration(125000), "2m 5s");
        }

        #[test]
        fn test_sanitize_filename() {
            assert_eq!(sanitize_filename("normal_file"), "normal_file");
            assert_eq!(sanitize_filename("file with spaces"), "file with spaces");
            assert_eq!(sanitize_filename("file<with>invalid:chars"), "file_with_invalid_chars");
            assert_eq!(sanitize_filename("file\"with|special*chars"), "file_with_special_chars");
            assert_eq!(sanitize_filename("file/with\\slashes"), "file_with_slashes");
            assert_eq!(sanitize_filename("file?with|question"), "file_with_question");
        }

        #[test]
        fn test_validate_project_name() {
            assert!(validate_project_name("Valid_Project").is_ok());
            assert!(validate_project_name("Project123").is_ok());
            assert!(validate_project_name("Project with spaces").is_ok());
            assert!(validate_project_name("Project-with-dashes").is_ok());

            assert!(validate_project_name("").is_err());
            assert!(validate_project_name("   ").is_err());
            assert!(validate_project_name("Project<With>Invalid").is_err());
            assert!(validate_project_name("Project:With:Colons").is_err());
            assert!(validate_project_name("Project\"With\"Quotes").is_err());
            assert!(validate_project_name("Project|With|Pipes").is_err());
            assert!(validate_project_name("Project?With?Questions").is_err());
            assert!(validate_project_name("Project*With*Stars").is_err());

            let long_name = "a".repeat(256);
            assert!(validate_project_name(&long_name).is_err());

            let max_length_name = "a".repeat(255);
            assert!(validate_project_name(&max_length_name).is_ok());
        }
    }


    mod integration_tests {
        use super::*;

        #[test]
        fn test_full_project_workflow() {
            let test_project = TestProject::new("TestProject");


            assert!(test_project.config.validate().is_ok());


            assert!(test_project.create_project_structure().is_ok());


            assert!(Path::new(&test_project.config.setup.name).exists());


            let config_path = "test_config.toml";
            assert!(Config::write_config(&test_project.config, config_path).is_ok());
            assert!(Path::new(config_path).exists());

            let loaded_config = Config::read_config(config_path);
            assert!(loaded_config.is_ok());
            let loaded_config = loaded_config.unwrap();
            assert_eq!(loaded_config.setup.name, test_project.config.setup.name);
        }

        #[test]
        fn test_query_execution() {
            let test_project = TestProject::new("QueryTestProject");
            test_project.create_project_structure().expect("Failed to create test structure");


            let root_result = query_root(&test_project.config, &QuerySettings::default());
            assert!(root_result.is_ok());

            if let Ok(QueryResult::RootResult(root)) = root_result {
                assert_eq!(root.project_name, "QueryTestProject");
                assert!(root.file_count >= 0);
            } else {
                panic!("Expected RootResult");
            }
        }

        #[test]
        fn test_argument_parsing_new_project() {
            let args = vec![
                "nanopm".to_string(),
                "new".to_string(),
                "-n".to_string(),
                "TestParseProject".to_string(),
                "-d".to_string(),
                "3".to_string(),
                "-c".to_string(),
                "4".to_string(),
                "-s".to_string(),
                "2".to_string(),
            ];

            let result = parse_args(args, false, &OperationType::New);

            if let ParsedReturn::Config(config) = result {
                assert_eq!(config.setup.name, "TestParseProject");
                assert_eq!(config.setup.days, 3);
                assert_eq!(config.setup.cameras, 4);
                assert_eq!(config.setup.sound_sources, 2);
            } else {
                panic!("Expected Config result");
            }
        }

        #[test]
        fn test_argument_parsing_query() {
            let test_project = TestProject::new("QueryParseTest");
            Config::write_config(&test_project.config, "config.toml").expect("Failed to write config");

            let args = vec![
                "nanopm".to_string(),
                "query".to_string(),
                "-g".to_string(),
                "-ss".to_string(),
                "-t".to_string(),
                "-rt".to_string(),
            ];

            let result = parse_args(args, true, &OperationType::Query);

            if let ParsedReturn::Query(query_info) = result {
                assert!(matches!(query_info.query, Query::General(SortType::BySize)));
                assert!(query_info.settings.record_timestamp);
                assert!(query_info.settings.include_runtime);
            } else {
                panic!("Expected Query result");
            }
        }

        #[test]
        fn test_default_query_behavior() {
            let test_project = TestProject::new("DefaultQueryTest");
            Config::write_config(&test_project.config, "config.toml").expect("Failed to write config");

            let args = vec![
                "nanopm".to_string(),
                "query".to_string(),
            ];

            let result = parse_args(args, true, &OperationType::Query);

            if let ParsedReturn::Query(query_info) = result {
                assert!(matches!(query_info.query, Query::General(SortType::ByDefaultOrder)));
            } else {
                panic!("Expected Query result with default general query");
            }
        }
    }


    mod error_tests {
        use super::*;

        #[test]
        fn test_config_error_display() {
            let io_error = ConfigError::IoError(std::io::Error::new(std::io::ErrorKind::NotFound, "File not found"));
            assert!(format!("{}", io_error).contains("IO error"));

            let parse_error = ConfigError::ParseError("Invalid syntax".to_string());
            assert!(format!("{}", parse_error).contains("Parsing error"));
        }

        #[test]
        fn test_query_error_display() {
            let query_error = QueryError::InvalidQuery("Bad query".to_string());
            assert!(format!("{}", query_error).contains("Invalid query"));

            let io_error = QueryError::IoError(std::io::Error::new(std::io::ErrorKind::PermissionDenied, "Access denied"));
            assert!(format!("{}", io_error).contains("IO error"));
        }

        #[test]
        fn test_invalid_config_handling() {
            let mut config = Config::new_config();
            config.setup.name = "".to_string();

            let validation_result = config.validate();
            assert!(validation_result.is_err());
            assert!(validation_result.unwrap_err().to_string().contains("empty"));
        }
    }


    mod performance_tests {
        use super::*;
        use std::time::Instant;

        #[test]
        fn test_config_creation_performance() {
            let start = Instant::now();
            for _ in 0..1000 {
                let _config = Config::new_config();
            }
            let duration = start.elapsed();


            assert!(duration.as_millis() < 100);
        }

        #[test]
        fn test_shorthand_conversion_performance() {
            let start = Instant::now();
            let test_sizes = vec![0, 1024, 1048576, 1073741824, 1099511627776];

            for _ in 0..10000 {
                for &size in &test_sizes {
                    let _ = to_shorthand(size);
                }
            }
            let duration = start.elapsed();


            assert!(duration.as_millis() < 100);
        }
    }


    mod edge_case_tests {
        use super::*;

        #[test]
        fn test_zero_values_handling() {
            assert_eq!(to_shorthand(0), "0B");

            let mut setup = ProjectSetup::new();
            setup.days = 0;
            assert!(setup.validate().is_err());
        }

        #[test]
        fn test_very_large_numbers() {
            let large_number = u64::MAX;
            let result = to_shorthand(large_number);
            assert!(result.contains("B"));
        }

        #[test]
        fn test_unicode_project_names() {
            let unicode_name = "Project_测试_🚀";
            let result = validate_project_name(unicode_name);
            assert!(result.is_ok());
        }

        #[test]
        fn test_empty_folder_lists() {
            let empty_query = Query::Folder(vec![], SortType::ByDefaultOrder);
            assert!(matches!(empty_query, Query::Folder(_, _)));
        }
    }
}


#[cfg(all(feature = "bench", test))]
mod benches {
    use super::*;
    use test::Bencher;

    #[bench]
    fn bench_config_creation(b: &mut Bencher) {
        b.iter(|| {
            let _config = Config::new_config();
       });
   }

   #[bench]
   fn bench_to_shorthand_conversion(b: &mut Bencher) {
       let test_values = vec![1024, 1048576, 1073741824, 1099511627776];
       b.iter(|| {
           for &value in &test_values {
               test::black_box(to_shorthand(value));
           }
       });
   }

   #[bench]
   fn bench_project_validation(b: &mut Bencher) {
       let setup = ProjectSetup::new();
       b.iter(|| {
           test::black_box(setup.validate());
       });
   }

   #[bench]
   fn bench_folder_creation(b: &mut Bencher) {
       b.iter(|| {
           let _folder = Folder::new("test_id", Some("parent_id"), "Test Folder");
       });
   }
}


pub mod test_utils {
   use super::*;
   use std::fs;
   use std::path::Path;


   pub fn create_test_project(name: &str, days: usize, cameras: usize, sound_sources: usize) -> Config {
       let mut config = Config::new_config();
       config.setup.name = name.to_string();
       config.setup.days = days;
       config.setup.cameras = cameras;
       config.setup.sound_sources = sound_sources;
       config
   }


   pub fn create_minimal_structure(project_name: &str) -> std::io::Result<()> {
       fs::create_dir(project_name)?;
       fs::create_dir(format!("{}/01_DOCUMENTATION", project_name))?;
       fs::create_dir(format!("{}/02_RUSHES", project_name))?;
       fs::create_dir(format!("{}/03_EXTERNAL", project_name))?;
       Ok(())
   }


   pub fn create_test_files(base_path: &Path, files: &[(&str, usize)]) -> std::io::Result<()> {
       for (filename, size) in files {
           let file_path = base_path.join(filename);
           if let Some(parent) = file_path.parent() {
               fs::create_dir_all(parent)?;
           }
           let content = vec![b'A'; *size];
           fs::write(file_path, content)?;
       }
       Ok(())
   }


   pub fn assert_directory_contains(dir_path: &Path, expected_files: &[&str]) {
       assert!(dir_path.exists(), "Directory {:?} does not exist", dir_path);

       for expected_file in expected_files {
           let file_path = dir_path.join(expected_file);
           assert!(file_path.exists(), "Expected file {:?} does not exist", file_path);
       }
   }


   pub fn count_files_recursive(dir_path: &Path) -> std::io::Result<usize> {
       use std::fs;

       let mut count = 0;
       for entry in fs::read_dir(dir_path)? {
           let entry = entry?;
           let path = entry.path();
           if path.is_file() {
               count += 1;
           } else if path.is_dir() {
               count += count_files_recursive(&path)?;
           }
       }
       Ok(count)
   }


   pub fn calculate_directory_size(dir_path: &Path) -> std::io::Result<u64> {
       use std::fs;

       let mut total_size = 0;
       for entry in fs::read_dir(dir_path)? {
           let entry = entry?;
           let path = entry.path();
           if path.is_file() {
               total_size += fs::metadata(&path)?.len();
           } else if path.is_dir() {
               total_size += calculate_directory_size(&path)?;
           }
       }
       Ok(total_size)
   }


   pub fn create_mock_query_info(query: Query) -> QueryInfo {
       QueryInfo {
           query,
           settings: QuerySettings::default(),
           config: Config::new_config(),
       }
   }


   pub fn validate_query_result(result: &QueryResult, expected_name: &str) {
       match result {
           QueryResult::GeneralResult(r) => {
               assert_eq!(r.folder_name, expected_name);
           }
           QueryResult::RootResult(r) => {
               assert_eq!(r.project_name, expected_name);
           }
           QueryResult::DayResult(r) => {
               assert!(r.day.contains(expected_name));
           }
           QueryResult::CamResult(r) => {
               assert!(r.camera.contains(expected_name));
           }
           QueryResult::SoundResult(r) => {
               assert!(r.sound_source.contains(expected_name));
           }
           QueryResult::FolderResult(_) => {

           }
       }
   }
}


#[cfg(test)]
pub mod mocks {
   use super::*;


   pub struct MockDirContent {
       pub files: Vec<String>,
       pub directories: Vec<String>,
       pub dir_size: u64,
   }

   impl MockDirContent {
       pub fn new() -> Self {
           MockDirContent {
               files: Vec::new(),
               directories: Vec::new(),
               dir_size: 0,
           }
       }

       pub fn with_files(mut self, files: Vec<String>) -> Self {
           self.files = files;
           self
       }

       pub fn with_directories(mut self, directories: Vec<String>) -> Self {
           self.directories = directories;
           self
       }

       pub fn with_size(mut self, size: u64) -> Self {
           self.dir_size = size;
           self
       }
   }


   pub fn mock_quiet_settings() -> QuerySettings {
       let mut settings = QuerySettings::default();
       settings.quiet = true;
       settings
   }

   pub fn mock_verbose_settings() -> QuerySettings {
       let mut settings = QuerySettings::default();
       settings.record_timestamp = true;
       settings.include_runtime = true;
       settings
   }

   pub fn mock_write_settings(output_name: &str) -> QuerySettings {
       let mut settings = QuerySettings::default();
       settings.write = true;
       settings.output_name = Some(output_name.to_string());
       settings
   }
}


#[cfg(test)]
pub mod property_tests {
   use super::*;
   use quickcheck::{quickcheck, TestResult};


   fn prop_sanitized_names_are_valid(name: String) -> TestResult {
       if name.is_empty() || name.len() > 1000 {
           return TestResult::discard();
       }

       let sanitized = crate::util::util::sanitize_filename(&name);
       let validation_result = crate::util::util::validate_project_name(&sanitized);

       TestResult::from_bool(validation_result.is_ok() || sanitized.trim().is_empty())
   }


   fn prop_shorthand_always_contains_b(size: u64) -> bool {
       let result = to_shorthand(size);
       result.contains('B')
   }


   fn prop_positive_integers_parse(n: u16) -> TestResult {
       if n == 0 {
           return TestResult::discard();
       }

       let result = parse_positive_integer(&n.to_string(), "test");
       TestResult::from_bool(result.is_ok() && result.unwrap() == n as usize)
   }

   #[test]
   fn run_property_tests() {
       quickcheck(prop_sanitized_names_are_valid as fn(String) -> TestResult);
       quickcheck(prop_shorthand_always_contains_b as fn(u64) -> bool);
       quickcheck(prop_positive_integers_parse as fn(u16) -> TestResult);
   }
}


#[cfg(test)]
mod regression_tests {
   use super::*;

   #[test]
   fn test_issue_5_sort_size_for_non_general_queries() {

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


       query_results.sort_by(|a, b| b.get_size_for_sorting().cmp(&a.get_size_for_sorting()));


       if let QueryResult::FolderResult(first) = &query_results[0] {
           assert_eq!(first.total_size_u64, 2097152);
       } else {
           panic!("Expected FolderResult");
       }
   }

   #[test]
   fn test_issue_7_flexible_folder_keys() {

       let folder = Folder::new("custom_id", Some("parent_custom"), "Custom Folder");
       assert_eq!(folder.id, "custom_id");
       assert_eq!(folder.parent_id, Some("parent_custom".to_string()));


       let folders = vec![
           Folder::new("z_folder", None, "Z Folder"),
           Folder::new("a_folder", Some("z_folder"), "A Folder"),
       ];

       assert_eq!(folders.len(), 2);
       assert_eq!(folders[1].parent_id, Some("z_folder".to_string()));
   }

   #[test]
   fn test_issue_8_default_query_type() {

       let args = vec![
           "nanopm".to_string(),
           "query".to_string(),
       ];

       let test_project = TestProject::new("DefaultTest");
       Config::write_config(&test_project.config, "config.toml").expect("Failed to write config");

       let result = parse_args(args, true, &OperationType::Query);

       if let ParsedReturn::Query(query_info) = result {
           assert!(matches!(query_info.query, Query::General(SortType::ByDefaultOrder)));
       } else {
           panic!("Expected Query result with default general query");
       }
   }

   #[test]
   fn test_issue_9_runtime_query_tag() {

       let mut settings = QuerySettings::default();
       settings.include_runtime = true;

       let result = GeneralResult {
           path: "test".to_string(),
           folder_name: "Test Folder".to_string(),
           file_count: 10,
           total_size: "1MB".to_string(),
           total_size_u64: 1048576,
           runtime_ms: Some(150),
       };


       let serialized = toml::to_string(&result).expect("Failed to serialize");
       assert!(serialized.contains("runtime_ms") || !settings.include_runtime);
   }

   #[test]
   fn test_circular_import_fix() {


       use crate::util::query;


       let query_info = QueryInfo::new_query_info();
       assert!(matches!(query_info.query, Query::None));
   }
}


#[cfg(test)]
mod doc_tests {


   use super::*;

   #[test]
   fn test_basic_usage_example() {

       let mut config = Config::new_config();
       config.setup.name = "My_Project".to_string();
       config.setup.days = 5;
       config.setup.cameras = 3;
       config.setup.sound_sources = 2;

       assert!(config.validate().is_ok());
       assert_eq!(config.setup.name, "My_Project");
       assert_eq!(config.setup.days, 5);
   }

   #[test]
   fn test_query_example() {

       let mut query_info = QueryInfo::new_query_info();
       query_info.query = Query::General(SortType::BySize);
       query_info.settings.quiet = true;

       assert!(matches!(query_info.query, Query::General(SortType::BySize)));
       assert!(query_info.settings.quiet);
   }

   #[test]
   fn test_shorthand_examples() {

       assert_eq!(to_shorthand(0), "0B");
       assert_eq!(to_shorthand(1024), "1KiB (1KB)");
       assert_eq!(to_shorthand(1048576), "1.0MiB (1.0MB)");
   }
}