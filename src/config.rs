//! Configuration constants for ytrs

// Filename templates use yt-dlp's fallback syntax: %(field,alternative|default)s

pub const FILENAME_PRIMARY: &str = "%(title)s - %(uploader,channel,creator|Unknown)s (%(height)sp, %(vcodec)s, %(extractor_key)s).%(ext)s";
pub const FILENAME_AUDIO_PRIMARY: &str =
    "%(title)s - %(uploader,channel,creator|Unknown)s (%(extractor_key)s).%(ext)s";
pub const FILENAME_VIDEO_ONLY_PRIMARY: &str = "%(title)s - %(uploader,channel,creator|Unknown)s (%(height)sp, %(vcodec)s, %(extractor_key)s, video-only).%(ext)s";

// Height capped at 2160p
pub const FORMAT_DEFAULT: &str = "bv*[height<=2160]+ba/b[height<=2160]";
pub const FORMAT_AUDIO_ONLY: &str = "ba/b";
pub const FORMAT_VIDEO_ONLY: &str = "bv[height<=2160]";

// VP9 > AV1 > H.264; Opus > FLAC > AAC > MP3; hdr:12 excludes Dolby Vision
pub const FORMAT_SORT_DEFAULT: &str = "res,fps,vcodec:vp9.2,vcodec:vp9,vcodec:av01,vcodec:hev1,vcodec:avc,hdr:12,acodec:opus,acodec:flac,acodec:aac,acodec:mp3,size";
pub const FORMAT_SORT_AUDIO: &str = "acodec:opus,acodec:flac,acodec:aac,acodec:mp3,abr";
pub const FORMAT_SORT_VIDEO: &str =
    "res,fps,vcodec:vp9.2,vcodec:vp9,vcodec:av01,vcodec:hev1,vcodec:avc,hdr:12,size";

pub const CONTAINER_VIDEO: &str = "webm/mkv/mp4";
#[allow(dead_code)]
pub const CONTAINER_AUDIO: &str = "opus/ogg/m4a";
pub const CONTAINER_SOCM: &str = "mp4";

pub const ARIA2C_ARGS: &str =
    "-x 8 -s 16 -k 2M --file-allocation=falloc --disk-cache=64M --enable-color=false";

pub const BATCH_SLEEP_THRESHOLD: usize = 10;
pub const BATCH_SLEEP_SECONDS: u64 = 5;
pub const REQUEST_SLEEP_SECONDS: f64 = 0.5;
pub const REQUIRED_DEPENDENCIES: &[&str] = &["yt-dlp", "aria2c", "ffmpeg"];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filename_template_has_fallbacks() {
        assert!(FILENAME_PRIMARY.contains("uploader,channel,creator|Unknown"));
    }

    #[test]
    fn test_format_sort_vp9_before_av1() {
        let vp9_pos = FORMAT_SORT_DEFAULT.find("vcodec:vp9");
        let av01_pos = FORMAT_SORT_DEFAULT.find("vcodec:av01");
        assert!(vp9_pos.unwrap() < av01_pos.unwrap());
    }

    #[test]
    fn test_format_sort_opus_preferred() {
        assert!(FORMAT_SORT_AUDIO.starts_with("acodec:opus"));
    }

    #[test]
    fn test_aria2c_conservative() {
        assert!(ARIA2C_ARGS.contains("-x 8"));
    }

    #[test]
    fn test_batch_threshold() {
        assert_eq!(BATCH_SLEEP_THRESHOLD, 10);
        assert_eq!(BATCH_SLEEP_SECONDS, 5);
    }
}
