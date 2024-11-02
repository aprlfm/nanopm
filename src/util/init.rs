#[derive(Eq, PartialEq, Debug)]
pub enum InitParams {
    None,
    ProjName,
    Days,
    Cameras,
    SoundSources,
}

impl InitParams {
    pub fn to_string(&self) -> String {
        match &self {
            InitParams::ProjName => String::from("ProjName"),
            InitParams::Days => String::from("Days"),
            InitParams::Cameras => String::from("Cameras"),
            InitParams::SoundSources => String::from("SoundSources"),
            other => panic!("Undefined InitParams variant \"{other:?}\" must be added to to_string() method (please report this) (ERROR CODE: 3)"),
        }
    }
}

#[derive(Debug)]
pub struct ProjectSetup {
    pub name : String,
    pub days : usize,
    pub cameras : usize,
    pub sound_sources: usize,
}

pub fn get_required_type(operation : InitParams, readable : bool) -> String {
    if readable {
        match operation {
            InitParams::ProjName => String::from("a String"),
            InitParams::Days => String::from("an integer"),
            InitParams::Cameras => String::from("an integer"),
            InitParams::SoundSources => String::from("an integer"),
            _ => String::from("No type found for this parameter (ERROR CODE: 2)")
        }
    } else {
        match operation {
            InitParams::ProjName => String::from("String"),
            InitParams::Days => String::from("usize"),
            InitParams::Cameras => String::from("usize"),
            InitParams::SoundSources => String::from("usize"),
            _ => String::from("invalid")
        }
    }
}