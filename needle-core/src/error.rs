use std::{
    error::Error,
    fmt::{self, Display, Formatter},
};

#[derive(Debug)]
pub enum NeedleError {
    // AppConfig
    InvalidPath,
    ConfigExists,
    ConfigNonExistant(Box<str>),

    // Surface related errors
    Lost,
    Outdated,
    OutOfMemory,
    Timeout,

    // Renderer related errors
    RemovedFromAtlas,
    ScreenResolutionChanged,
}

impl Display for NeedleError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let msg = match self {
            Self::InvalidPath => "AppConfig | Invalid Path".to_string(),
            Self::ConfigExists => "AppConfig | Config Already Exists".to_string(),
            Self::ConfigNonExistant(path) => {
                format!("AppConfig | Config File Doesn't exist ({})", path)
            }
            Self::Lost => "Surface | Lost".to_string(),
            Self::Outdated => "Surface | Outdated".to_string(),
            Self::OutOfMemory => "Surface | Out Of Memory".to_string(),
            Self::Timeout => "Surface | Timeout".to_string(),
            Self::RemovedFromAtlas => "Renderer | Removed From Atlas".to_string(),
            Self::ScreenResolutionChanged => "Renderer | Screen Resolution Changed".to_string(),
        };

        writeln!(f, "[ERROR]: {}", msg)
    }
}

impl Error for NeedleError {}

pub type NeedleErr<T> = Result<T, NeedleError>;
