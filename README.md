# ytrs

> **Note**: This is a personal utility designed for my specific workflow and preferences. It's an opinionated wrapper that streamlines my daily usage.

`ytrs` is a high-performance, asynchronous Rust wrapper for `yt-dlp`. It orchestrates media downloads from thousands of supported sites with a focus on quality, speed, and reliability. By leveraging Rust's concurrency model and `aria2c`'s multi-connection capabilities, it ensures optimal throughput while maintaining system stability.

## Features

- **Opinionated Quality Defaults**: Automatically prioritizes AV1 and VP9 codecs for maximum fidelity, falling back gracefully when unavailable.
- **High-Performance Engine**: Integrates with `aria2c` to utilize 16 connections per download, significantly accelerating transfer speeds.
- **Concurrency Control**: Built on the Tokio runtime with semaphore-based concurrency limiting to safely manage parallel batch downloads.
- **Social Media Mode**: specialized `--socm` flag that forces MP4/H.264/AAC formats for maximum compatibility with sharing platforms.
- **Robust Signal Handling**: Implements clean shutdown procedures via `signal-hook`, ensuring no zombie processes or corrupted files upon interruption (Ctrl+C).
- **Smart Output Naming**: Enforces a standardized naming convention including title, ID, resolution, FPS, and codecs for easy library management.

## Requirements

- **Rust**: 1.75+ (2024 edition)
- **Dependencies**: `yt-dlp` and `aria2c` must be installed and available in your PATH.

## Installation

Build the binary in release mode for optimal performance:

```bash
cargo build --release
```

The resulting binary will be located at `target/release/ytrs`.

## Usage

`ytrs` simplifies complex `yt-dlp` commands into intuitive flags.

### Basic Download
Download a single video with default high-quality settings (AV1/VP9):
```bash
ytrs "https://youtube.com/watch?v=..."
```

### Batch Processing
Download multiple URLs in parallel (default: 2 concurrent downloads):
```bash
ytrs "URL1" "URL2" "URL3"
```

### Social Media Compatibility
Force legacy compatible formats (MP4 container, H.264 video, AAC audio) for easy sharing:
```bash
ytrs --socm "https://twitter.com/user/status/..."
```

### Browser Cookies
Load cookies from a specific browser to access authenticated content:
```bash
ytrs --cookies-from firefox "URL"
```

## Configuration

| Flag | Description | Default |
|------|-------------|---------|
| `-c, --codec <CODEC>` | Preferred video codec (`av1` or `vp9`). Ignored if `--socm` is active. | `av1` |
| `-d, --destination <PATH>` | Specify output directory or full file path. | Current Dir |
| `-p, --parallel <N>` | Number of concurrent downloads in batch mode. | `2` |
| `--socm` | Enable Social Media Mode (MP4/H.264/AAC). | `false` |
| `--cookies-from <BROWSER>` | Source browser for cookies (e.g., `firefox`, `chrome`). | None |

## License

This project is licensed under the terms specified in the [LICENSE](LICENSE) file.
