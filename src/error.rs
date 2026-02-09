//! Error types with human-readable messages

use thiserror::Error;

#[derive(Debug, Error)]
pub enum YtrsError {
    #[error("Dependency '{0}' is not installed or not found in PATH")]
    MissingDependency(String),

    #[error("Download failed for '{url}': {reason}")]
    DownloadFailed { url: String, reason: String },

    #[error("yt-dlp process error: {0}")]
    #[allow(dead_code)]
    ProcessError(String),

    #[error("No valid URLs provided")]
    NoValidUrls,

    #[error("{0} download(s) failed")]
    PartialFailure(usize),

    #[error("Invalid mode combination: {0}")]
    InvalidModeCombo(String),

    #[error("Semaphore closed unexpectedly")]
    SemaphoreClosed,

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, YtrsError>;

pub fn extract_error_reason(stderr: &str, exit_code: Option<i32>) -> String {
    let patterns = [
        ("Video unavailable", "Video is unavailable or private"),
        ("Private video", "Video is private"),
        ("This video is private", "Video is private"),
        ("429", "Rate limited by server - try again later"),
        (
            "Too Many Requests",
            "Rate limited by server - try again later",
        ),
        ("403", "Access forbidden - may require cookies"),
        ("Forbidden", "Access forbidden - may require cookies"),
        ("404", "Video not found"),
        ("Not Found", "Video not found"),
        (
            "Sign in to confirm your age",
            "Age-restricted - requires account cookies",
        ),
        (
            "age-restricted",
            "Age-restricted - requires account cookies",
        ),
        (
            "members-only",
            "Members-only content - requires membership cookies",
        ),
        (
            "Join this channel",
            "Members-only content - requires membership cookies",
        ),
        ("no longer available", "Video is no longer available"),
        ("has been removed", "Video has been removed"),
        ("copyright", "Video removed due to copyright claim"),
        ("blocked", "Video is blocked in your region"),
        ("country", "Video is not available in your country"),
        ("No video formats", "No downloadable video formats found"),
        (
            "Requested format not available",
            "Requested format not available",
        ),
        ("is not a valid URL", "Invalid URL format"),
        ("Unsupported URL", "Website not supported by yt-dlp"),
        ("Unable to extract", "Failed to extract video information"),
        ("Connection refused", "Connection refused by server"),
        ("timed out", "Connection timed out"),
        ("Name or service not known", "DNS resolution failed"),
    ];

    for (pattern, message) in patterns {
        if stderr.contains(pattern) {
            return message.to_string();
        }
    }

    match exit_code {
        Some(1) => "General error occurred".to_string(),
        Some(2) => "Invalid arguments provided".to_string(),
        Some(code) => format!("yt-dlp exited with code {code}"),
        None => "Process terminated unexpectedly".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_error_reason_video_unavailable() {
        let stderr = "ERROR: Video unavailable. This video is no longer available.";
        assert_eq!(
            extract_error_reason(stderr, Some(1)),
            "Video is unavailable or private"
        );
    }

    #[test]
    fn test_extract_error_reason_rate_limited() {
        let stderr = "ERROR: HTTP Error 429: Too Many Requests";
        assert_eq!(
            extract_error_reason(stderr, Some(1)),
            "Rate limited by server - try again later"
        );
    }

    #[test]
    fn test_extract_error_reason_age_restricted() {
        let stderr = "ERROR: Sign in to confirm your age";
        assert_eq!(
            extract_error_reason(stderr, Some(1)),
            "Age-restricted - requires account cookies"
        );
    }

    #[test]
    fn test_extract_error_reason_unsupported() {
        let stderr = "ERROR: Unsupported URL: https://example.com/video";
        assert_eq!(
            extract_error_reason(stderr, Some(1)),
            "Website not supported by yt-dlp"
        );
    }

    #[test]
    fn test_extract_error_reason_fallback() {
        let stderr = "Some unknown error occurred";
        assert_eq!(
            extract_error_reason(stderr, Some(42)),
            "yt-dlp exited with code 42"
        );
    }

    #[test]
    fn test_extract_error_reason_no_exit_code() {
        let stderr = "Unknown error";
        assert_eq!(
            extract_error_reason(stderr, None),
            "Process terminated unexpectedly"
        );
    }

    #[test]
    fn test_error_display() {
        let err = YtrsError::DownloadFailed {
            url: "https://example.com".to_string(),
            reason: "Video is private".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "Download failed for 'https://example.com': Video is private"
        );
    }
}
