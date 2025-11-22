# ytrs

Rust wrapper for yt-dlp with optimized download settings.

## Requirements

- `yt-dlp`
- `aria2c`
- Rust 1.75+

## Installation

```bash
cargo build --release
```

Binary: `target/release/ytrs`

## Usage

```bash
# Single download
ytrs "https://youtube.com/watch?v=..."

# Custom destination
ytrs -d ~/Downloads/ "URL"

# Batch download (2 parallel)
ytrs "URL1" "URL2" "URL3"

# VP9 codec preference
ytrs -c vp9 "URL"

# Social media compatibility (MP4/H.264)
ytrs --socm "URL"

# Browser cookies
ytrs --cookies-from firefox "URL"
```

## Options

```
-c, --codec <av1|vp9>        Video codec preference [default: av1]
-d, --destination <PATH>     Download destination
-p, --parallel <N>           Parallel downloads [default: 2]
--socm                       Social media mode (MP4/H.264/AAC)
--cookies-from <BROWSER>     Load cookies from browser
```

## Features

- AV1/VP9 codec selection with format sorting
- aria2c integration (16 connections, 32 streams)
- Concurrent downloads with semaphore control
- Signal handling (Ctrl+C safe)
- URL validation and deduplication

## Output Format

Default: `%(title)s [%(id)s][%(height)sp][%(fps)sfps][%(vcodec)s][%(acodec)s].%(ext)s`

Merge: MKV (default) | MP4 (social media mode)

## License

See LICENSE file.
