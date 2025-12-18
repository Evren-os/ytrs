use std::borrow::Cow;
use std::path::Path;

use crate::config::{
    ARIA2C_ARGS, DEFAULT_FILENAME_PATTERN, DEFAULT_MERGE_FORMAT, FORMAT_QUALITY, FORMAT_SOCM,
    SOCM_MERGE_FORMAT, SOCM_POSTPROCESSOR_ARGS, VP9_FORMAT_SORT,
};

#[derive(Default)]
pub struct YtDlpArgs<'a> {
    pub destination_path: Option<&'a Path>,
    pub cookies_from: Option<&'a str>,
    pub socm: bool,
}

pub fn build_ytdlp_args<'a>(url: &'a str, args: &YtDlpArgs<'a>) -> Vec<Cow<'a, str>> {
    let output_template: Cow<'a, str> = match args.destination_path {
        Some(dest) if dest.is_dir() => Cow::Owned(
            dest.join(DEFAULT_FILENAME_PATTERN)
                .to_string_lossy()
                .into_owned(),
        ),
        Some(dest) => Cow::Owned(dest.to_string_lossy().into_owned()),
        None => Cow::Borrowed(DEFAULT_FILENAME_PATTERN),
    };

    let capacity = if args.socm { 18 } else { 16 };
    let mut result: Vec<Cow<'a, str>> = Vec::with_capacity(capacity);

    result.extend([
        Cow::Borrowed("--remote-components"),
        Cow::Borrowed("ejs:github"),
        Cow::Borrowed("--prefer-free-formats"),
        Cow::Borrowed("--format-sort-force"),
        Cow::Borrowed("--no-mtime"),
        Cow::Borrowed("--output"),
        output_template,
        Cow::Borrowed("--external-downloader"),
        Cow::Borrowed("aria2c"),
        Cow::Borrowed("--external-downloader-args"),
        Cow::Borrowed(ARIA2C_ARGS),
    ]);

    if let Some(cookies) = args.cookies_from {
        result.push(Cow::Borrowed("--cookies-from-browser"));
        result.push(Cow::Borrowed(cookies));
    }

    if args.socm {
        build_socm_args(&mut result);
    } else {
        build_quality_args(&mut result);
    }

    result.push(Cow::Borrowed(url));
    result
}

fn build_quality_args<'a>(result: &mut Vec<Cow<'a, str>>) {
    result.extend([
        Cow::Borrowed("--merge-output-format"),
        Cow::Borrowed(DEFAULT_MERGE_FORMAT),
        Cow::Borrowed("--format"),
        Cow::Borrowed(FORMAT_QUALITY),
        Cow::Borrowed("--format-sort"),
        Cow::Borrowed(VP9_FORMAT_SORT),
    ]);
}

fn build_socm_args<'a>(result: &mut Vec<Cow<'a, str>>) {
    result.extend([
        Cow::Borrowed("--merge-output-format"),
        Cow::Borrowed(SOCM_MERGE_FORMAT),
        Cow::Borrowed("--format"),
        Cow::Borrowed(FORMAT_SOCM),
        Cow::Borrowed("--format-sort"),
        Cow::Borrowed(VP9_FORMAT_SORT),
        Cow::Borrowed("--postprocessor-args"),
        Cow::Borrowed(SOCM_POSTPROCESSOR_ARGS),
    ]);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_ytdlp_args_vp9() {
        let args = YtDlpArgs::default();
        let result = build_ytdlp_args("https://example.com", &args);
        assert!(result.iter().any(|s| s == "--format-sort"));
        assert!(result.iter().any(|s| s == VP9_FORMAT_SORT));
        assert!(result.iter().any(|s| s == "https://example.com"));
    }

    #[test]
    fn test_build_ytdlp_args_socm() {
        let args = YtDlpArgs {
            socm: true,
            ..Default::default()
        };
        let result = build_ytdlp_args("https://example.com", &args);
        assert!(result.iter().any(|s| s == "mp4"));
        assert!(result.iter().any(|s| s.contains("libx264")));
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
}
