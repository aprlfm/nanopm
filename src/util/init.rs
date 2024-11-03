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
    Size,
    FileCount,
    //...,
}

#[derive(Eq, PartialEq, Debug)]
pub enum OperationType {
    New,
    Update,
    Query,
}

impl InitParams {
    pub fn to_string(&self) -> String {
        match &self {
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
    pub fn to_string(&self) -> String {
        match &self {
            QueryParams::None => String::from("None"),
            QueryParams::Size => String::from("Size"),
            QueryParams::FileCount => String::from("FileCount"),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ProjectSetup {
    pub name : String,
    #[serde(skip_serializing)]
    pub deadname : Option<String>,
    pub days : usize,
    pub cameras : usize,
    pub sound_sources: usize,
    #[serde(skip_serializing, default)]
    pub clean_project: bool,
}

pub fn new_project_setup() -> ProjectSetup{
    ProjectSetup{
        name : String::from("Untitled"),
        deadname: None,
        days : 2,
        cameras : 2,
        sound_sources : 1,
        clean_project : false,
    }
}

pub fn get_required_type_init(operation : InitParams, readable : bool) -> String {
    if readable {
        match operation {
            InitParams::ProjName => String::from("a String"),
            InitParams::DeadName => String::from("a String"),
            InitParams::Days => String::from("an integer"),
            InitParams::Cameras => String::from("an integer"),
            InitParams::SoundSources => String::from("an integer"),
            _ => String::from("No type found for this parameter (ERROR CODE: 2)")
        }
    } else {
        match operation {
            InitParams::ProjName => String::from("String"),
            InitParams::DeadName => String::from("String"),
            InitParams::Days => String::from("usize"),
            InitParams::Cameras => String::from("usize"),
            InitParams::SoundSources => String::from("usize"),
            _ => String::from("invalid")
        }
    }
}

pub fn get_required_type_query(operation : QueryParams, readable : bool) -> String {
    if readable {
        match operation {
            QueryParams::Size => String::from("a String"),
            QueryParams::FileCount => String::from("a String"),
            _ => String::from("No type found for this parameter (ERROR CODE: 2)")
        }
    } else {
        match operation {
            QueryParams::Size => String::from("String"),
            QueryParams::FileCount => String::from("String"),
            _ => String::from("invalid")
        }
    }
}