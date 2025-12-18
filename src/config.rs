pub const DEFAULT_FILENAME_PATTERN: &str =
    "%(title)s [%(id)s][%(height)sp][%(fps)sfps][%(vcodec)s][%(acodec)s].%(ext)s";

pub const DEFAULT_MERGE_FORMAT: &str = "mkv";

pub const VP9_FORMAT_SORT: &str =
    "res,fps,vcodec:vp9.2,vcodec:vp9,vcodec:hev1,acodec:opus,acodec:aac";

pub const SOCM_MERGE_FORMAT: &str = "mp4";

pub const ARIA2C_ARGS: &str = "aria2c:-x 16 -s 32 -k 1M --disk-cache=128M --enable-color=false";

pub const FORMAT_QUALITY: &str = "bv*[height<=2160]+ba/bv*[height<=2160]";

pub const FORMAT_SOCM: &str = "bv*[height<=1080]+ba/bv*[height<=1080]";

pub const SOCM_POSTPROCESSOR_ARGS: &str =
    "ffmpeg:-c:v libx264 -preset slow -crf 18 -c:a aac -b:a 192k -movflags +faststart";
