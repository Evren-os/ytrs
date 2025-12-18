use thiserror::Error;

#[derive(Debug, Error)]
pub enum YtrsError {
    #[error("Dependency '{0}' is not installed or not found in PATH")]
    MissingDependency(String),

    #[error("yt-dlp failed with exit code: {0:?}")]
    YtDlpFailed(Option<i32>),

    #[error("No valid URLs provided")]
    NoValidUrls,

    #[error("{0} downloads failed")]
    PartialFailure(usize),

    #[error("Semaphore closed unexpectedly")]
    SemaphoreClosed,

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, YtrsError>;
