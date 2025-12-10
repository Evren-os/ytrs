pub const DEFAULT_FILENAME_PATTERN: &str =
    "%(title)s [%(id)s][%(height)sp][%(fps)sfps][%(vcodec)s][%(acodec)s].%(ext)s";

pub const DEFAULT_MERGE_FORMAT: &str = "mkv";

pub const VP9_FORMAT_SORT: &str =
    "res,fps,vcodec:vp9.2,vcodec:vp9,vcodec:hev1,acodec:opus,acodec:aac";

pub const MAX_RESOLUTION: u32 = 2160;

pub const SOCM_MERGE_FORMAT: &str = "mp4";

pub const SOCM_MAX_HEIGHT: u32 = 1080;

pub const SOCM_VIDEO_CRF: u32 = 18;

pub const SOCM_AUDIO_BITRATE: &str = "192k";

pub const ARIA2C_ARGS: &str = "-x 16 -s 32 -k 1M --disk-cache=128M --enable-color=false";
