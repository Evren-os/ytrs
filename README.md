# ytrs

> **Note**: This is a personal utility designed for my specific workflow and preferences. It's an opinionated wrapper that streamlines my daily usage.

`ytrs` is a high-performance, asynchronous Rust wrapper for `yt-dlp`. It orchestrates media downloads from thousands of supported sites with a focus on quality, speed, and reliability. By leveraging Rust's concurrency model and `aria2c`'s multi-connection capabilities, it ensures optimal throughput while maintaining system stability.

## Features

- **VP9-First Quality**: Automatically prioritizes VP9 > AV1 > H.264 for maximum fidelity with broad hardware compatibility. Resolution capped at 4K.
- **High-Performance Engine**: Integrates with `aria2c` using conservative 8-connection settings to maximize speed while avoiding rate-limiting.
- **Concurrency Control**: Built on the Tokio runtime with semaphore-based concurrency limiting to safely manage parallel batch downloads.
- **Social Media Optimization**: Platform-specific presets for WhatsApp, Discord, Instagram, Messenger, Signal, and Telegram with tuned encoding settings.
- **Audio-Only Mode**: Download just the audio in highest quality Opus format.
- **Video-Only Mode**: Download video without audio track for custom muxing.
- **Smart Rate Limiting**: Auto-detects large batches (>10 URLs) and applies sleep intervals to prevent server throttling.
- **Robust Signal Handling**: Implements clean shutdown procedures via `signal-hook`, ensuring no zombie processes or corrupted files upon interruption (Ctrl+C).
- **Human-Readable Errors**: Parses yt-dlp errors and presents clear, actionable messages instead of cryptic exit codes.
- **Smart Output Naming**: Standardized naming convention: `Title - Author (resolution, codec, platform).ext`

## Requirements

- **Rust**: 1.85+ (2024 edition)
- **Dependencies**: `yt-dlp`, `aria2c`, and `ffmpeg` must be installed and available in your PATH.

## Installation

Build the binary in release mode for optimal performance:

```bash
cargo build --release
```

The resulting binary will be located at `target/release/ytrs`.

## Usage

`ytrs` simplifies complex `yt-dlp` commands into intuitive flags.

### Basic Download
Download a single video with default high-quality settings (VP9 > AV1 > H.264):
```bash
ytrs "https://youtube.com/watch?v=..."
```

### Batch Processing
Download multiple URLs in parallel (default: 2 concurrent downloads):
```bash
ytrs "URL1" "URL2" "URL3"
```

### Audio Only
Download only the audio in highest quality Opus format:
```bash
ytrs -a "https://youtube.com/watch?v=..."
```

### Video Only
Download only the video (no audio track):
```bash
ytrs -v "https://youtube.com/watch?v=..."
```

### Social Media Optimization
Optimize downloads for specific platforms with tuned encoding settings:

```bash
# WhatsApp: 16MB limit, 1080p, H.264/AAC, CRF 23
ytrs --socm whatsapp "https://twitter.com/user/status/..."

# Discord: 25MB limit, 1080p, higher quality (CRF 20)
ytrs --socm discord "https://youtube.com/watch?v=..."

# Instagram: 15MB limit, 720p optimized
ytrs --socm instagram "https://tiktok.com/@user/video/..."

# Messenger: 25MB limit, 1080p
ytrs --socm messenger "https://youtube.com/watch?v=..."

# Signal: 100MB limit, highest quality (CRF 18)
ytrs --socm signal "https://youtube.com/watch?v=..."

# Telegram: 2GB limit, 4K support, highest quality (CRF 18)
ytrs --socm telegram "https://youtube.com/watch?v=..."
```

Short aliases are also supported: `wa`, `dc`, `ig`, `fb`, `sig`, `tg`

### Browser Cookies
Load cookies from a specific browser to access authenticated content:
```bash
ytrs --cookies-from firefox "URL"
```

### Custom Destination
Specify output directory:
```bash
ytrs -d ~/Videos "URL"
```

### Parallel Downloads
Control concurrency for batch downloads:
```bash
ytrs -p 4 "URL1" "URL2" "URL3" "URL4"
```

## Configuration

| Flag | Description | Default |
|------|-------------|---------|
| `-d, --destination <PATH>` | Specify output directory or full file path. | Current Dir |
| `-p, --parallel <N>` | Number of concurrent downloads in batch mode. | `2` |
| `-a, --audio` | Download audio only (Opus format). | `false` |
| `-v, --video` | Download video only (no audio). | `false` |
| `--socm <PLATFORM>` | Social media optimization target. | None |
| `--cookies-from <BROWSER>` | Source browser for cookies (e.g., `firefox`, `chrome`). | None |

## Codec Strategy

This tool uses a priority-based codec selection:

| Priority | Video Codec | Notes |
|----------|-------------|-------|
| 1st | VP9.2 | HDR capable |
| 2nd | VP9 | Excellent quality/compatibility |
| 3rd | AV1 | Future-proof, fallback if no VP9 |
| 4th | HEVC | Good quality, wider device support |
| 5th | H.264 | Universal compatibility |

Audio codec priority: **Opus > FLAC > AAC > MP3**

## Social Media Presets

| Platform | Max Size | Max Resolution | CRF | Audio Bitrate |
|----------|----------|----------------|-----|---------------|
| WhatsApp | 16MB | 1080p | 23 | 128k |
| Discord | 25MB | 1080p | 20 | 160k |
| Instagram | 15MB | 720p | 23 | 128k |
| Messenger | 25MB | 1080p | 20 | 160k |
| Signal | 100MB | 1080p | 18 | 192k |
| Telegram | 2GB | 2160p (4K) | 18 | 192k |

## Rate Limiting

When downloading more than 10 URLs, ytrs automatically adds sleep intervals between downloads to prevent server rate-limiting (YouTube enforces ~300 videos/hour for guests).

## Error Handling

ytrs parses yt-dlp errors and provides human-readable messages:

- "Video is unavailable or private"
- "Rate limited by server - try again later"
- "Access forbidden - may require cookies"
- "Age-restricted - requires account cookies"
- "Website not supported by yt-dlp"

## License

This project is licensed under the terms specified in the [LICENSE](LICENSE) file.