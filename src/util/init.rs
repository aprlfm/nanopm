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