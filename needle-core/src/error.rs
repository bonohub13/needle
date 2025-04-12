use crate::Position;
use std::error::Error as StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum NeedleError {
    // AppConfig
    #[error("AppConfig | Invalid path")]
    InvalidPath,
    #[error("AppConfig | Config already exists")]
    ConfigExists,
    #[error("AppConfig | Config file doesn't exist ({0})")]
    ConfigNonExistant(Box<str>),
    #[error("AppConfig | Text position is invalid. Must be corners. ({0})")]
    InvalidFpsTextPosition(Position),
    #[error("AppConfig | Text position for FPS and time is overlapping")]
    TextPositionOverlapping,

    // Surface related errors
    #[error("Surface | Lost")]
    Lost,
    #[error("Surface | Outdated")]
    Outdated,
    #[error("Surface | Out of memory")]
    OutOfMemory,
    #[error("Surface | Timeout")]
    Timeout,

    // Renderer related errors
    #[error("Renderer | Removed from atlas")]
    RemovedFromAtlas,
    #[error("Renderer | Screen resolution changed")]
    ScreenResolutionChanged,
    #[error("Renderer | Buffer without bind group/bind group layout has been registered")]
    InvalidBufferRegistration,
    #[error("Renderer | Buffer without bind group/bind group layout has been registered ({0})")]
    RendererUpdateFailure(Box<dyn StdError>),

    // cURL related errors
    #[error("URL | Invalid URL format detected")]
    InvalidURLFormat,
    #[error("URL | Error detected in callback function")]
    CallbackError,
    #[error("URL | Failed to download shader")]
    ShaderDownloadFailure,
    #[error("URL | Failed to write to file")]
    WriteError,

    // Clock related errors
    #[error("Clock | Failed to start countup/countdown timer")]
    TimerStartFailure,

    // Other errors
    #[error("Other | Initialization error detected")]
    InitializationError,
    #[error("Other | Unknown error has been detected! Please file an issue to the repository if possible.")]
    Other,
}

unsafe impl Send for NeedleError {}
unsafe impl Sync for NeedleError {}

pub type NeedleErr<T> = Result<T, NeedleError>;
