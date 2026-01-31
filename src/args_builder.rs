//! yt-dlp argument builder for different download modes
//!
//! This module constructs the complete yt-dlp command-line arguments
//! based on the download mode (Default, `AudioOnly`, `VideoOnly`, `SocialMedia`)

use std::borrow::Cow;
use std::path::Path;

use crate::cli::SocialMediaTarget;
use crate::config::{
    ARIA2C_ARGS, BATCH_SLEEP_SECONDS, CONTAINER_SOCM, CONTAINER_VIDEO, FILENAME_AUDIO_PRIMARY,
    FILENAME_PRIMARY, FILENAME_VIDEO_ONLY_PRIMARY, FORMAT_AUDIO_ONLY, FORMAT_DEFAULT,
    FORMAT_SORT_AUDIO, FORMAT_SORT_DEFAULT, FORMAT_SORT_VIDEO, FORMAT_VIDEO_ONLY,
    REQUEST_SLEEP_SECONDS,
};
use crate::mode::DownloadMode;

/// Arguments for building yt-dlp command
#[derive(Default)]
pub struct YtDlpArgs<'a> {
    /// Output destination
    pub destination_path: Option<&'a Path>,

    /// Browser to load cookies from
    pub cookies_from: Option<&'a str>,

    /// Download mode determining format and post-processing
    pub mode: DownloadMode,

    /// Whether to apply rate limiting for large batches
    pub apply_rate_limit: bool,
}

/// Build complete yt-dlp arguments for a given URL and configuration
pub fn build_ytdlp_args<'a>(url: &'a str, args: &YtDlpArgs<'a>) -> Vec<Cow<'a, str>> {
    let output_template = build_output_template(args.mode, args.destination_path);

    // Estimate capacity based on mode
    let capacity = match args.mode {
        DownloadMode::SocialMedia(_) => 24,
        _ => 20,
    };
    let mut result: Vec<Cow<'a, str>> = Vec::with_capacity(capacity);

    // Core arguments (all modes)
    result.extend([
        // Enable remote JS components for YouTube support
        Cow::Borrowed("--remote-components"),
        Cow::Borrowed("ejs:github"),
        Cow::Borrowed("--prefer-free-formats"),
        Cow::Borrowed("--format-sort-force"),
        Cow::Borrowed("--no-mtime"),
        Cow::Borrowed("--output"),
        Cow::Owned(output_template),
        Cow::Borrowed("--external-downloader"),
        Cow::Borrowed("aria2c"),
        Cow::Borrowed("--external-downloader-args"),
        Cow::Borrowed(ARIA2C_ARGS),
    ]);

    // Cookies
    if let Some(cookies) = args.cookies_from {
        result.push(Cow::Borrowed("--cookies-from-browser"));
        result.push(Cow::Borrowed(cookies));
    }

    // Rate limiting for large batches
    if args.apply_rate_limit {
        result.extend([
            Cow::Borrowed("--sleep-requests"),
            Cow::Owned(REQUEST_SLEEP_SECONDS.to_string()),
            Cow::Borrowed("--sleep-interval"),
            Cow::Owned(BATCH_SLEEP_SECONDS.to_string()),
        ]);
    }

    // Mode-specific arguments
    match &args.mode {
        DownloadMode::Default => build_default_args(&mut result),
        DownloadMode::AudioOnly => build_audio_args(&mut result),
        DownloadMode::VideoOnly => build_video_args(&mut result),
        DownloadMode::SocialMedia(target) => build_socm_args(&mut result, *target),
    }

    // URL (always last)
    result.push(Cow::Borrowed(url));

    result
}

/// Build output template based on mode and destination
fn build_output_template(mode: DownloadMode, destination: Option<&Path>) -> String {
    let template = match mode {
        DownloadMode::AudioOnly => FILENAME_AUDIO_PRIMARY,
        DownloadMode::VideoOnly => FILENAME_VIDEO_ONLY_PRIMARY,
        DownloadMode::SocialMedia(_) | DownloadMode::Default => FILENAME_PRIMARY,
    };

    match destination {
        Some(dest) if dest.is_dir() => dest.join(template).to_string_lossy().into_owned(),
        Some(dest) => dest.to_string_lossy().into_owned(),
        None => template.to_string(),
    }
}

/// Arguments for default maximum quality mode
fn build_default_args(result: &mut Vec<Cow<'_, str>>) {
    result.extend([
        // Container preference: WebM > MKV > MP4
        Cow::Borrowed("--merge-output-format"),
        Cow::Borrowed(CONTAINER_VIDEO),
        // Format selection: best video + best audio
        Cow::Borrowed("--format"),
        Cow::Borrowed(FORMAT_DEFAULT),
        // Format sorting: VP9 > AV1 > H.264
        Cow::Borrowed("--format-sort"),
        Cow::Borrowed(FORMAT_SORT_DEFAULT),
    ]);
}

/// Arguments for audio-only mode
fn build_audio_args(result: &mut Vec<Cow<'_, str>>) {
    result.extend([
        // Extract audio
        Cow::Borrowed("-x"),
        // Container preference: Opus > OGG > M4A
        Cow::Borrowed("--audio-format"),
        Cow::Borrowed("opus"),
        // Format selection: best audio
        Cow::Borrowed("--format"),
        Cow::Borrowed(FORMAT_AUDIO_ONLY),
        // Format sorting: Opus > FLAC > AAC
        Cow::Borrowed("--format-sort"),
        Cow::Borrowed(FORMAT_SORT_AUDIO),
    ]);
}

/// Arguments for video-only mode
fn build_video_args(result: &mut Vec<Cow<'_, str>>) {
    result.extend([
        // Container preference: WebM > MKV > MP4
        Cow::Borrowed("--merge-output-format"),
        Cow::Borrowed(CONTAINER_VIDEO),
        // Format selection: best video only
        Cow::Borrowed("--format"),
        Cow::Borrowed(FORMAT_VIDEO_ONLY),
        // Format sorting: VP9 > AV1 > H.264
        Cow::Borrowed("--format-sort"),
        Cow::Borrowed(FORMAT_SORT_VIDEO),
    ]);
}

/// Arguments for social media optimization mode
fn build_socm_args(result: &mut Vec<Cow<'_, str>>, target: SocialMediaTarget) {
    // Get platform-specific configuration
    let format_selector = target.format_selector();
    let format_sort = target.format_sort();
    let pp_args = target.postprocessor_args();

    result.extend([
        // Container: MP4 for maximum compatibility
        Cow::Borrowed("--merge-output-format"),
        Cow::Borrowed(CONTAINER_SOCM),
        // Remux to MP4 if possible
        Cow::Borrowed("--remux-video"),
        Cow::Borrowed("mp4"),
        // Format selection with resolution cap
        Cow::Borrowed("--format"),
        Cow::Owned(format_selector),
        // Format sorting: prefer H.264 source to avoid re-encoding
        Cow::Borrowed("--format-sort"),
        Cow::Owned(format_sort),
        // Post-processor args for re-encoding when necessary
        Cow::Borrowed("--postprocessor-args"),
        Cow::Owned(pp_args),
    ]);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_build_ytdlp_args_default() {
        let args = YtDlpArgs::default();
        let result = build_ytdlp_args("https://example.com", &args);

        assert!(result.iter().any(|s| s == "--format-sort"));
        assert!(result.iter().any(|s| s.contains("vp9")));
        assert!(result.iter().any(|s| s == "https://example.com"));
        assert!(result.iter().any(|s| s == "--merge-output-format"));
    }

    #[test]
    fn test_build_ytdlp_args_audio() {
        let args = YtDlpArgs {
            mode: DownloadMode::AudioOnly,
            ..Default::default()
        };
        let result = build_ytdlp_args("https://example.com", &args);

        assert!(result.iter().any(|s| s == "-x"));
        assert!(result.iter().any(|s| s == "--audio-format"));
        assert!(result.iter().any(|s| s == "opus"));
    }

    #[test]
    fn test_build_ytdlp_args_video() {
        let args = YtDlpArgs {
            mode: DownloadMode::VideoOnly,
            ..Default::default()
        };
        let result = build_ytdlp_args("https://example.com", &args);

        assert!(result.iter().any(|s| s == "bv"));
    }

    #[test]
    fn test_build_ytdlp_args_socm_discord() {
        let args = YtDlpArgs {
            mode: DownloadMode::SocialMedia(SocialMediaTarget::Discord),
            ..Default::default()
        };
        let result = build_ytdlp_args("https://example.com", &args);

        assert!(result.iter().any(|s| s == "mp4"));
        assert!(result.iter().any(|s| s.contains("libx264")));
        assert!(result.iter().any(|s| s.contains("height<=1080")));
    }

    #[test]
    fn test_build_ytdlp_args_socm_instagram() {
        let args = YtDlpArgs {
            mode: DownloadMode::SocialMedia(SocialMediaTarget::Instagram),
            ..Default::default()
        };
        let result = build_ytdlp_args("https://example.com", &args);

        // Instagram should cap at 720p
        assert!(result.iter().any(|s| s.contains("height<=720")));
    }

    #[test]
    fn test_build_ytdlp_args_with_destination() {
        let path = Path::new("/tmp");
        let args = YtDlpArgs {
            destination_path: Some(path),
            ..Default::default()
        };
        let result = build_ytdlp_args("https://example.com", &args);

        assert!(result.iter().any(|s| s.contains("/tmp")));
    }

    #[test]
    fn test_build_ytdlp_args_with_cookies() {
        let args = YtDlpArgs {
            cookies_from: Some("firefox"),
            ..Default::default()
        };
        let result = build_ytdlp_args("https://example.com", &args);

        assert!(result.iter().any(|s| s == "--cookies-from-browser"));
        assert!(result.iter().any(|s| s == "firefox"));
    }

    #[test]
    fn test_build_ytdlp_args_with_rate_limit() {
        let args = YtDlpArgs {
            apply_rate_limit: true,
            ..Default::default()
        };
        let result = build_ytdlp_args("https://example.com", &args);

        assert!(result.iter().any(|s| s == "--sleep-requests"));
        assert!(result.iter().any(|s| s == "--sleep-interval"));
    }

    #[test]
    fn test_url_always_last() {
        let args = YtDlpArgs::default();
        let result = build_ytdlp_args("https://example.com", &args);

        assert_eq!(result.last().unwrap(), "https://example.com");
    }
}
