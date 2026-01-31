//! Configuration constants for ytrs
//!
//! All static configuration values are defined here, including:
//! - Filename templates
//! - Format selection strings
//! - Format sorting priorities
//! - Container preferences
//! - External downloader arguments
//! - Rate limiting thresholds

// Filename Templates

/// Primary filename template for video downloads
///
/// Format: "Video Title - Author (2160p, vp9, youtube).webm"
/// Uses yt-dlp's fallback syntax: %(field,alternative|default)s
pub const FILENAME_PRIMARY: &str =
    "%(title)s - %(uploader,channel,creator|Unknown)s (%(height)sp, %(vcodec)s, %(extractor_key)s).%(ext)s";

/// Primary filename template for audio-only downloads
///
/// Format: "Audio Title - Author (youtube).opus"
pub const FILENAME_AUDIO_PRIMARY: &str =
    "%(title)s - %(uploader,channel,creator|Unknown)s (%(extractor_key)s).%(ext)s";

/// Primary filename template for video-only downloads
///
/// Format: "Video Title - Author (2160p, vp9, youtube, video-only).webm"
pub const FILENAME_VIDEO_ONLY_PRIMARY: &str =
    "%(title)s - %(uploader,channel,creator|Unknown)s (%(height)sp, %(vcodec)s, %(extractor_key)s, video-only).%(ext)s";

// Format Selection

/// Default format selection: best video + best audio, fallback to best combined
pub const FORMAT_DEFAULT: &str = "bv*+ba/b";

/// Audio-only format selection: best audio stream
pub const FORMAT_AUDIO_ONLY: &str = "ba/b";

/// Video-only format selection: best video stream
pub const FORMAT_VIDEO_ONLY: &str = "bv";

// Format Sorting

/// Default format sort priority
///
/// Codec priority: VP9 > AV1 > H.264
/// - vp9.2: VP9 Profile 2 (HDR capable)
/// - vp9: VP9 Profile 0/1
/// - av01: AV1 (fallback if no VP9)
/// - hev1: HEVC/H.265
/// - avc: H.264
///
/// Audio priority: Opus > FLAC > AAC > MP3
///
/// HDR: hdr:12 excludes Dolby Vision
pub const FORMAT_SORT_DEFAULT: &str =
    "res,fps,vcodec:vp9.2,vcodec:vp9,vcodec:av01,vcodec:hev1,vcodec:avc,hdr:12,acodec:opus,acodec:flac,acodec:aac,acodec:mp3,size";

/// Audio-only format sort priority
///
/// Prioritizes Opus for best quality-to-size ratio
pub const FORMAT_SORT_AUDIO: &str = "acodec:opus,acodec:flac,acodec:aac,acodec:mp3,abr";

/// Video-only format sort priority
///
/// Same video codec priority as default
pub const FORMAT_SORT_VIDEO: &str =
    "res,fps,vcodec:vp9.2,vcodec:vp9,vcodec:av01,vcodec:hev1,vcodec:avc,hdr:12,size";

// Containers

/// Preferred container for video downloads
///
/// WebM preferred for VP9/AV1, with fallbacks for other codecs
pub const CONTAINER_VIDEO: &str = "webm/mkv/mp4";

/// Preferred container for audio-only downloads
///
/// Opus in OGG container preferred, with fallbacks
pub const CONTAINER_AUDIO: &str = "opus/ogg/m4a";

/// Container for social media exports
///
/// MP4 for maximum compatibility across platforms
pub const CONTAINER_SOCM: &str = "mp4";

// External Downloader (aria2c)

/// aria2c arguments for external downloading
pub const ARIA2C_ARGS: &str =
    "-x 8 -s 16 -k 2M --file-allocation=falloc --disk-cache=64M --enable-color=false";

// Rate Limiting

/// URL count threshold for auto-applying sleep between downloads
pub const BATCH_SLEEP_THRESHOLD: usize = 10;

/// Sleep duration in seconds
pub const BATCH_SLEEP_SECONDS: u64 = 5;

/// Sleep between requests
pub const REQUEST_SLEEP_SECONDS: f64 = 0.5;

/// Required external dependencies
pub const REQUIRED_DEPENDENCIES: &[&str] = &["yt-dlp", "aria2c", "ffmpeg"];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filename_template_has_fallbacks() {
        // Verify the template uses yt-dlp's fallback syntax
        assert!(FILENAME_PRIMARY.contains("uploader,channel,creator|Unknown"));
    }

    #[test]
    fn test_format_sort_vp9_before_av1() {
        // Verify VP9 comes before AV1 in the sort order
        let vp9_pos = FORMAT_SORT_DEFAULT.find("vcodec:vp9");
        let av01_pos = FORMAT_SORT_DEFAULT.find("vcodec:av01");
        assert!(vp9_pos.unwrap() < av01_pos.unwrap());
    }

    #[test]
    fn test_format_sort_opus_preferred() {
        // Verify Opus is first in audio sort
        assert!(FORMAT_SORT_AUDIO.starts_with("acodec:opus"));
    }

    #[test]
    fn test_aria2c_conservative() {
        // Verify aria2c uses 8 connections
        assert!(ARIA2C_ARGS.contains("-x 8"));
    }

    #[test]
    fn test_batch_threshold() {
        assert_eq!(BATCH_SLEEP_THRESHOLD, 10);
        assert_eq!(BATCH_SLEEP_SECONDS, 5);
    }
}