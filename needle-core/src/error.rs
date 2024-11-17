#[derive(Debug)]
pub enum NeedleError {
    // Surface related errors
    Lost,
    Outdated,
    OutOfMemory,
    Timeout,

    // Renderer related errors
    RemovedFromAtlas,
    ScreenResolutionChanged,
}

pub type NeedleErr<T> = Result<T, NeedleError>;
