//! yt-dlp argument builder for different download modes

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

#[derive(Default)]
pub struct YtDlpArgs<'a> {
    pub destination_path: Option<&'a Path>,
    pub cookies_from: Option<&'a str>,
    pub mode: DownloadMode,
    pub apply_rate_limit: bool,
}

pub fn build_ytdlp_args<'a>(url: &'a str, args: &YtDlpArgs<'a>) -> Vec<Cow<'a, str>> {
    let output_template = build_output_template(args.mode, args.destination_path);

    let capacity = match args.mode {
        DownloadMode::SocialMedia(_) => 24,
        _ => 20,
    };
    let mut result: Vec<Cow<'a, str>> = Vec::with_capacity(capacity);

    result.extend([
        Cow::Borrowed("--ignore-config"),
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

    if let Some(cookies) = args.cookies_from {
        result.push(Cow::Borrowed("--cookies-from-browser"));
        result.push(Cow::Borrowed(cookies));
    }

    if args.apply_rate_limit {
        result.extend([
            Cow::Borrowed("--sleep-requests"),
            Cow::Owned(REQUEST_SLEEP_SECONDS.to_string()),
            Cow::Borrowed("--sleep-interval"),
            Cow::Owned(BATCH_SLEEP_SECONDS.to_string()),
        ]);
    }

    match &args.mode {
        DownloadMode::Default => build_default_args(&mut result),
        DownloadMode::AudioOnly => build_audio_args(&mut result),
        DownloadMode::VideoOnly => build_video_args(&mut result),
        DownloadMode::SocialMedia(target) => build_socm_args(&mut result, *target),
    }

    result.push(Cow::Borrowed(url));

    result
}

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

fn build_default_args(result: &mut Vec<Cow<'_, str>>) {
    result.extend([
        Cow::Borrowed("--merge-output-format"),
        Cow::Borrowed(CONTAINER_VIDEO),
        Cow::Borrowed("--format"),
        Cow::Borrowed(FORMAT_DEFAULT),
        Cow::Borrowed("--format-sort"),
        Cow::Borrowed(FORMAT_SORT_DEFAULT),
    ]);
}

fn build_audio_args(result: &mut Vec<Cow<'_, str>>) {
    result.extend([
        Cow::Borrowed("-x"),
        Cow::Borrowed("--audio-format"),
        Cow::Borrowed("opus"),
        Cow::Borrowed("--format"),
        Cow::Borrowed(FORMAT_AUDIO_ONLY),
        Cow::Borrowed("--format-sort"),
        Cow::Borrowed(FORMAT_SORT_AUDIO),
    ]);
}

fn build_video_args(result: &mut Vec<Cow<'_, str>>) {
    result.extend([
        Cow::Borrowed("--merge-output-format"),
        Cow::Borrowed(CONTAINER_VIDEO),
        Cow::Borrowed("--format"),
        Cow::Borrowed(FORMAT_VIDEO_ONLY),
        Cow::Borrowed("--format-sort"),
        Cow::Borrowed(FORMAT_SORT_VIDEO),
    ]);
}

fn build_socm_args(result: &mut Vec<Cow<'_, str>>, target: SocialMediaTarget) {
    let format_selector = target.format_selector();
    let format_sort = target.format_sort();
    let pp_args = target.postprocessor_args();

    result.extend([
        Cow::Borrowed("--merge-output-format"),
        Cow::Borrowed(CONTAINER_SOCM),
        Cow::Borrowed("--remux-video"),
        Cow::Borrowed("mp4"),
        Cow::Borrowed("--format"),
        Cow::Owned(format_selector),
        Cow::Borrowed("--format-sort"),
        Cow::Owned(format_sort),
        Cow::Borrowed("--postprocessor-args"),
        Cow::Owned(pp_args),
    ]);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    const ALL_MODES: [DownloadMode; 9] = [
        DownloadMode::Default,
        DownloadMode::AudioOnly,
        DownloadMode::VideoOnly,
        DownloadMode::SocialMedia(SocialMediaTarget::WhatsApp),
        DownloadMode::SocialMedia(SocialMediaTarget::Discord),
        DownloadMode::SocialMedia(SocialMediaTarget::Instagram),
        DownloadMode::SocialMedia(SocialMediaTarget::Messenger),
        DownloadMode::SocialMedia(SocialMediaTarget::Signal),
        DownloadMode::SocialMedia(SocialMediaTarget::Telegram),
    ];

    const NEVER_EMITTED_FLAGS: [&str; 8] = [
        "--exec",
        "--write-link",
        "--write-url-link",
        "--write-desktop-link",
        "--netrc-cmd",
        "--write-subs",
        "--write-thumbnail",
        "--compat-options",
    ];

    fn assert_no_advisory_flags(result: &[Cow<'_, str>]) {
        for window in result.windows(2) {
            let pair = format!("{} {}", window[0], window[1]);
            assert_ne!(pair, "--compat-options allow-unsafe-ext");
            assert_ne!(pair, "--external-downloader curl");
        }
        for arg in result {
            let token = arg.as_ref();
            for flag in NEVER_EMITTED_FLAGS {
                assert_ne!(token, flag, "advisory-triggering flag {flag} emitted");
                assert!(
                    !token.starts_with(&format!("{flag}=")),
                    "advisory-triggering flag {flag} emitted"
                );
            }
            assert_ne!(token, "--external-downloader=curl");
            assert_ne!(token, "curl", "curl external downloader emitted");
            assert_ne!(token, "allow-unsafe-ext", "allow-unsafe-ext emitted");
        }
    }

    #[test]
    fn test_external_downloader_is_aria2c() {
        for mode in ALL_MODES {
            let args = YtDlpArgs {
                mode,
                ..Default::default()
            };
            let result = build_ytdlp_args("https://example.com", &args);

            let downloader_pos = result
                .iter()
                .position(|s| s == "--external-downloader")
                .unwrap_or_else(|| panic!("mode {mode:?} missing --external-downloader"));
            assert_eq!(result[downloader_pos + 1], "aria2c");
        }
    }

    fn args_for_mode(mode: DownloadMode) -> YtDlpArgs<'static> {
        YtDlpArgs {
            mode,
            cookies_from: Some("firefox"),
            apply_rate_limit: true,
            ..Default::default()
        }
    }

    #[test]
    fn test_config_isolation_flag_present_in_all_modes() {
        for mode in ALL_MODES {
            let args = args_for_mode(mode);
            let result = build_ytdlp_args("https://example.com", &args);

            assert!(
                result.iter().any(|s| s == "--ignore-config"),
                "mode {mode:?} missing --ignore-config"
            );
        }
    }

    #[test]
    fn test_never_emits_advisory_triggering_flags() {
        for mode in ALL_MODES {
            let args = args_for_mode(mode);
            let result = build_ytdlp_args("https://example.com", &args);

            assert_no_advisory_flags(&result);
        }
    }

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

        assert!(result.iter().any(|s| s.contains("bv[height<=2160]")));
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
