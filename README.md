# ytrs

> **Note**: This is a personal utility designed for my specific workflow and preferences. It's an opinionated wrapper that streamlines my daily usage.

`ytrs` is a high-performance, asynchronous Rust wrapper for `yt-dlp`. It orchestrates media downloads from thousands of supported sites with a focus on quality, speed, and reliability. By leveraging Rust's concurrency model and `aria2c`'s multi-connection capabilities, it ensures optimal throughput while maintaining system stability.

## Features

- **VP9-First Quality**: Automatically prioritizes VP9 codec for maximum fidelity with broad hardware compatibility.
- **High-Performance Engine**: Integrates with `aria2c` to utilize 16 connections per download, significantly accelerating transfer speeds.
- **Concurrency Control**: Built on the Tokio runtime with semaphore-based concurrency limiting to safely manage parallel batch downloads.
- **Quality-Preserving Social Media Mode**: The `--socm` flag re-encodes to H.264/AAC with CRF 18 (visually lossless) for maximum platform compatibility while preserving quality.
- **Robust Signal Handling**: Implements clean shutdown procedures via `signal-hook`, ensuring no zombie processes or corrupted files upon interruption (Ctrl+C).
- **Smart Output Naming**: Enforces a standardized naming convention including title, ID, resolution, FPS, and codecs for easy library management.

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
Download a single video with default high-quality settings (VP9):
```bash
ytrs "https://youtube.com/watch?v=..."
```

### Batch Processing
Download multiple URLs in parallel (default: 2 concurrent downloads):
```bash
ytrs "URL1" "URL2" "URL3"
```

### Social Media Compatibility
Re-encode to compatible formats (MP4 container, H.264 video, AAC audio) with quality preservation:
```bash
ytrs --socm "https://twitter.com/user/status/..."
```

This mode:
- Downloads the best available VP9 stream
- Re-encodes using `ffmpeg` with CRF 18 (visually lossless, ~10% quality/size reduction)
- Outputs to MP4 container with `+faststart` for streaming compatibility
- Limits resolution to 1080p for platform requirements

### Browser Cookies
Load cookies from a specific browser to access authenticated content:
```bash
ytrs --cookies-from firefox "URL"
```

## Configuration

| Flag | Description | Default |
|------|-------------|---------|
| `-d, --destination <PATH>` | Specify output directory or full file path. | Current Dir |
| `-p, --parallel <N>` | Number of concurrent downloads in batch mode. | `2` |
| `--socm` | Enable Social Media Mode (quality-preserving H.264/AAC re-encoding). | `false` |
| `--cookies-from <BROWSER>` | Source browser for cookies (e.g., `firefox`, `chrome`). | None |

## Codec Strategy

This tool exclusively uses **VP9** codec for all downloads. AV1 is not supported due to hardware decoding constraints on systems without discrete GPUs. VP9 provides an excellent balance of quality and compatibility:

- **VP9.2** (HDR) prioritized when available
- **VP9** as primary fallback
- **HEVC** as secondary fallback
- **Opus** audio preferred, with **AAC** as fallback

## License

This project is licensed under the terms specified in the [LICENSE](LICENSE) file.
