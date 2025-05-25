use serde::{Deserialize, Serialize};

#[derive(Eq, PartialEq, Debug)]
pub enum InitParams {
    None,
    ProjName,
    DeadName,
    Days,
    Cameras,
    SoundSources,
}

#[derive(Eq, PartialEq, Debug)]
pub enum QueryParams {
    None,
    OutputDir,
    Folder,
}

#[derive(Eq, PartialEq, Debug)]
pub enum OperationType {
    None,
    New,
    Update,
    Query,
}

impl InitParams {
    pub fn _to_string(&self) -> String {
        match self {
            InitParams::None => String::from("None"),
            InitParams::ProjName => String::from("ProjName"),
            InitParams::DeadName => String::from("DeadName"),
            InitParams::Days => String::from("Days"),
            InitParams::Cameras => String::from("Cameras"),
            InitParams::SoundSources => String::from("SoundSources"),
        }
    }
}

impl QueryParams {
    pub fn _to_string(&self) -> String {
        match self {
            QueryParams::None => String::from("None"),
            QueryParams::OutputDir => String::from("Output Directory"),
            QueryParams::Folder => String::from("Folder"),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ProjectSetup {
    pub name: String,
    #[serde(skip_serializing)]
    pub deadname: Option<String>,
    pub days: usize,
    pub cameras: usize,
    pub sound_sources: usize,
    #[serde(skip_serializing, default)]
    pub clean_project: bool,
}

impl Default for ProjectSetup {
    fn default() -> Self {
        Self::new()
    }
}

impl ProjectSetup {
    pub fn new() -> Self {
        ProjectSetup {
            name: String::from("Untitled_Project"),
            deadname: None,
            days: 2,
            cameras: 2,
            sound_sources: 1,
            clean_project: false,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() {
            return Err("Project name cannot be empty".to_string());
        }
        if self.days == 0 {
            return Err("Number of days must be greater than 0".to_string());
        }
        if self.cameras == 0 {
            return Err("Number of cameras must be greater than 0".to_string());
        }
        if self.sound_sources == 0 {
            return Err("Number of sound sources must be greater than 0".to_string());
        }
        Ok(())
    }
}

pub fn new_project_setup() -> ProjectSetup {
    ProjectSetup::new()
}

pub fn get_required_type_init(operation: InitParams, readable: bool) -> String {
    if readable {
        match operation {
            InitParams::ProjName => String::from("a String"),
            InitParams::DeadName => String::from("a String"),
            InitParams::Days => String::from("a positive integer"),
            InitParams::Cameras => String::from("a positive integer"),
            InitParams::SoundSources => String::from("a positive integer"),
            InitParams::None => String::from("None"),
        }
    } else {
        match operation {
            InitParams::ProjName => String::from("String"),
            InitParams::DeadName => String::from("String"),
            InitParams::Days => String::from("usize"),
            InitParams::Cameras => String::from("usize"),
            InitParams::SoundSources => String::from("usize"),
            InitParams::None => String::from("None"),
        }
    }
}

pub fn get_required_type_query(operation: QueryParams, readable: bool) -> String {
    if readable {
        match operation {
            QueryParams::Folder => String::from("a String"),
            QueryParams::OutputDir => String::from("a String"),
            QueryParams::None => String::from("None"),
        }
    } else {
        match operation {
            QueryParams::Folder => String::from("String"),
            QueryParams::OutputDir => String::from("String"),
            QueryParams::None => String::from("None"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_setup_validation() {
        let mut setup = ProjectSetup::new();
        assert!(setup.validate().is_ok());

        setup.name = "".to_string();
        assert!(setup.validate().is_err());

        setup.name = "Valid".to_string();
        setup.days = 0;
        assert!(setup.validate().is_err());

        setup.days = 1;
        setup.cameras = 0;
        assert!(setup.validate().is_err());

        setup.cameras = 1;
        setup.sound_sources = 0;
        assert!(setup.validate().is_err());
    }

    #[test]
    fn test_default_project_setup() {
        let setup = ProjectSetup::default();
        assert_eq!(setup.name, "Untitled_Project");
        assert_eq!(setup.days, 2);
        assert_eq!(setup.cameras, 2);
        assert_eq!(setup.sound_sources, 1);
        assert!(!setup.clean_project);
        assert!(setup.deadname.is_none());
    }

    #[test]
    fn test_get_required_type_functions() {
        assert_eq!(
            get_required_type_init(InitParams::Days, true),
            "a positive integer"
        );
        assert_eq!(get_required_type_init(InitParams::Days, false), "usize");
        assert_eq!(
            get_required_type_query(QueryParams::Folder, true),
            "a String"
        );
    }
}
