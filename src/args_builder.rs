use std::path::Path;

use crate::config::{
    ARIA2C_ARGS, DEFAULT_FILENAME_PATTERN, DEFAULT_MERGE_FORMAT, MAX_RESOLUTION,
    SOCM_AUDIO_BITRATE, SOCM_MAX_HEIGHT, SOCM_MERGE_FORMAT, SOCM_VIDEO_CRF, VP9_FORMAT_SORT,
};

pub struct YtDlpArgs {
    pub destination_path: Option<String>,
    pub cookies_from: Option<String>,
    pub socm: bool,
}

pub fn build_ytdlp_args(url: &str, args: &YtDlpArgs) -> Vec<String> {
    let output_template = match args.destination_path {
        Some(ref dest) => {
            let path = Path::new(dest);
            if path.is_dir() {
                path.join(DEFAULT_FILENAME_PATTERN)
                    .to_string_lossy()
                    .to_string()
            } else {
                dest.clone()
            }
        }
        None => DEFAULT_FILENAME_PATTERN.to_string(),
    };

    let mut result = vec![
        "--prefer-free-formats".to_string(),
        "--format-sort-force".to_string(),
        "--no-mtime".to_string(),
        "--output".to_string(),
        output_template,
        "--external-downloader".to_string(),
        "aria2c".to_string(),
        "--external-downloader-args".to_string(),
        ARIA2C_ARGS.to_string(),
    ];

    if let Some(ref cookies) = args.cookies_from {
        result.push("--cookies-from-browser".to_string());
        result.push(cookies.clone());
    }

    if args.socm {
        build_socm_args(&mut result);
    } else {
        build_quality_args(&mut result);
    }

    result.push(url.to_string());
    result
}

fn build_quality_args(result: &mut Vec<String>) {
    let format_string = format!(
        "bv*[height<={}]+ba/bv*[height<={}]",
        MAX_RESOLUTION, MAX_RESOLUTION
    );

    result.push("--merge-output-format".to_string());
    result.push(DEFAULT_MERGE_FORMAT.to_string());
    result.push("--format".to_string());
    result.push(format_string);
    result.push("--format-sort".to_string());
    result.push(VP9_FORMAT_SORT.to_string());
}

fn build_socm_args(result: &mut Vec<String>) {
    let format_string = format!(
        "bv*[height<={}]+ba/bv*[height<={}]",
        SOCM_MAX_HEIGHT, SOCM_MAX_HEIGHT
    );

    result.push("--merge-output-format".to_string());
    result.push(SOCM_MERGE_FORMAT.to_string());
    result.push("--format".to_string());
    result.push(format_string);
    result.push("--format-sort".to_string());
    result.push(VP9_FORMAT_SORT.to_string());

    result.push("--postprocessor-args".to_string());
    result.push(format!(
        "ffmpeg:-c:v libx264 -preset slow -crf {} -c:a aac -b:a {} -movflags +faststart",
        SOCM_VIDEO_CRF, SOCM_AUDIO_BITRATE
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_ytdlp_args_vp9() {
        let args = YtDlpArgs {
            destination_path: None,
            cookies_from: None,
            socm: false,
        };
        let result = build_ytdlp_args("https://example.com", &args);
        assert!(result.contains(&"--format-sort".to_string()));
        assert!(result.contains(&VP9_FORMAT_SORT.to_string()));
        assert!(result.contains(&"https://example.com".to_string()));
    }

    #[test]
    fn test_build_ytdlp_args_socm() {
        let args = YtDlpArgs {
            destination_path: None,
            cookies_from: None,
            socm: true,
        };
        let result = build_ytdlp_args("https://example.com", &args);
        assert!(result.contains(&"mp4".to_string()));
        assert!(result.iter().any(|s| s.contains("libx264")));
    }

    #[test]
    fn test_build_ytdlp_args_with_destination() {
        let args = YtDlpArgs {
            destination_path: Some("/tmp".to_string()),
            cookies_from: None,
            socm: false,
        };
        let result = build_ytdlp_args("https://example.com", &args);
        assert!(result.iter().any(|s| s.contains("/tmp")));
    }

    #[test]
    fn test_build_ytdlp_args_with_cookies() {
        let args = YtDlpArgs {
            destination_path: None,
            cookies_from: Some("firefox".to_string()),
            socm: false,
        };
        let result = build_ytdlp_args("https://example.com", &args);
        assert!(result.contains(&"--cookies-from-browser".to_string()));
        assert!(result.contains(&"firefox".to_string()));
    }
}
