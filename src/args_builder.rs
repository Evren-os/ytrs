use std::path::Path;

const DEFAULT_FILENAME_PATTERN: &str =
    "%(title)s [%(id)s][%(height)sp][%(fps)sfps][%(vcodec)s][%(acodec)s].%(ext)s";
const DEFAULT_MERGE_FORMAT: &str = "mkv";
const CODEC_AV1: &str = "av1";
const CODEC_VP9: &str = "vp9";
const SOCM_FORMAT: &str =
    "bv*[vcodec^=avc][height<=1080]+ba[acodec^=mp4a]/b[vcodec^=avc][height<=1080]";
const SOCM_MERGE_FORMAT: &str = "mp4";

pub struct YtDlpArgs {
    pub codec_pref: String,
    pub destination_path: Option<String>,
    pub cookies_from: Option<String>,
    pub socm: bool,
}

pub fn build_ytdlp_args(url: &str, args: &YtDlpArgs) -> Vec<String> {
    let output_template = if let Some(ref dest) = args.destination_path {
        let path = Path::new(dest);
        if path.is_dir() {
            path.join(DEFAULT_FILENAME_PATTERN)
                .to_string_lossy()
                .to_string()
        } else {
            dest.clone()
        }
    } else {
        DEFAULT_FILENAME_PATTERN.to_string()
    };

    let mut result = vec![
        "--remote-components".to_string(),
        "ejs:github".to_string(),
        "--prefer-free-formats".to_string(),
        "--format-sort-force".to_string(),
        "--no-mtime".to_string(),
        "--output".to_string(),
        output_template,
        "--external-downloader".to_string(),
        "aria2c".to_string(),
        "--external-downloader-args".to_string(),
        "-x 16 -s 32 -k 1M --disk-cache=128M --enable-color=false".to_string(),
    ];

    if let Some(ref cookies) = args.cookies_from {
        result.push("--cookies-from-browser".to_string());
        result.push(cookies.clone());
    }

    if args.socm {
        result.push("--merge-output-format".to_string());
        result.push(SOCM_MERGE_FORMAT.to_string());
        result.push("--format".to_string());
        result.push(SOCM_FORMAT.to_string());
    } else {
        let max_height = 2160;
        let format_string = format!("bv*[height<={}]+ba/bv*[height<={}]", max_height, max_height);

        let sort_string = match args.codec_pref.to_lowercase().as_str() {
            CODEC_AV1 => "res,fps,vcodec:av01,vcodec:vp9.2,vcodec:vp9,vcodec:hev1,acodec:opus",
            CODEC_VP9 => "res,fps,vcodec:vp9,vcodec:vp9.2,vcodec:av01,vcodec:hev1,acodec:opus",
            _ => panic!("Invalid codec preference. Use 'av1' or 'vp9'."),
        };

        result.push("--merge-output-format".to_string());
        result.push(DEFAULT_MERGE_FORMAT.to_string());
        result.push("--format".to_string());
        result.push(format_string);
        result.push("--format-sort".to_string());
        result.push(sort_string.to_string());
    }

    result.push(url.to_string());
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_ytdlp_args_av1() {
        let args = YtDlpArgs {
            codec_pref: "av1".to_string(),
            destination_path: None,
            cookies_from: None,
            socm: false,
        };
        let result = build_ytdlp_args("https://example.com", &args);
        assert!(result.contains(&"--format-sort".to_string()));
        assert!(result.contains(&"https://example.com".to_string()));
    }

    #[test]
    fn test_build_ytdlp_args_socm() {
        let args = YtDlpArgs {
            codec_pref: "av1".to_string(),
            destination_path: None,
            cookies_from: None,
            socm: true,
        };
        let result = build_ytdlp_args("https://example.com", &args);
        assert!(result.contains(&"mp4".to_string()));
    }
}
