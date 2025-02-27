use crate::Position;
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
    InvalidFpsTextPosition(Position),
    TextPositionOverlapping,

    // Surface related errors
    Lost,
    Outdated,
    OutOfMemory,
    Timeout,

    // Renderer related errors
    RemovedFromAtlas,
    ScreenResolutionChanged,
    InvalidBufferRegistration,

    // cURL related errors
    InvalidURLFormat,
    CallbackError,
    ShaderDownloadFailure,
    WriteError,

    // Other errors
    Other,
}

impl Display for NeedleError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let msg = match self {
            Self::InvalidPath => "AppConfig | Invalid path".to_string(),
            Self::ConfigExists => "AppConfig | Config already exists".to_string(),
            Self::ConfigNonExistant(path) => {
                format!("AppConfig | Config file doesn't exist ({})", path)
            }
            Self::InvalidFpsTextPosition(pos) => {
                format!(
                    "AppConfig | Text position is invalid. Must be corners. ({})",
                    pos
                )
            }
            Self::TextPositionOverlapping => {
                "AppConfig | Text position for FPS and time is overlapping".to_string()
            }
            Self::Lost => "Surface | Lost".to_string(),
            Self::Outdated => "Surface | Outdated".to_string(),
            Self::OutOfMemory => "Surface | Out of memory".to_string(),
            Self::Timeout => "Surface | Timeout".to_string(),
            Self::RemovedFromAtlas => "Renderer | Removed from atlas".to_string(),
            Self::ScreenResolutionChanged => "Renderer | Screen resolution changed".to_string(),
            Self::InvalidBufferRegistration => {
                "Renderer | Buffer without bind group/bind group layout has been registered"
                    .to_string()
            }
            Self::InvalidURLFormat => {
                "URL | Invalid URL format detected".to_string()
            }
            Self::CallbackError => {
                "URL | Error detected in callback function".to_string()
            }
            Self::ShaderDownloadFailure => {
                "URL | Failed to download shader".to_string()
            }
            Self::WriteError => {
                "URL | Failed to write to file".to_string()
            }
            Self::Other => {
                "Other | Unknown error has been detected! Please file an issue to the repository if possible.".to_string()
            }
        };

        writeln!(f, "[ERROR]: {}", msg)
    }
}

impl Error for NeedleError {}

pub type NeedleErr<T> = Result<T, NeedleError>;
