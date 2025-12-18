mod args_builder;
mod config;
mod dependencies;
mod downloader;
mod error;
mod url_validator;

use std::num::NonZeroUsize;
use std::path::PathBuf;

use clap::Parser;
use colored::Colorize;

use crate::dependencies::check_dependencies;
use crate::downloader::{download_batch, download_single};
use crate::error::{Result, YtrsError};
use crate::url_validator::validate_url;

#[derive(Parser)]
#[command(
    name = "ytrs",
    about = "A high-quality wrapper for yt-dlp with VP9 codec optimization.",
    long_about = "Downloads media with maximum quality VP9 codec. Supports batch mode with multiple URLs."
)]
struct Cli {
    #[arg(
        short = 'd',
        long,
        help = "Download destination. Can be a directory or a full file path."
    )]
    destination: Option<PathBuf>,

    #[arg(
        long,
        help = "Load cookies from the specified browser (e.g., firefox, chrome)."
    )]
    cookies_from: Option<String>,

    #[arg(
        long,
        help = "Optimize for social media compatibility (MP4, H.264/AAC). Uses quality-preserving re-encoding."
    )]
    socm: bool,

    #[arg(
        short = 'p',
        long,
        default_value = "2",
        help = "Number of parallel downloads for batch mode."
    )]
    parallel: NonZeroUsize,

    #[arg(required = true, help = "URL(s) to download")]
    urls: Vec<String>,
}

fn run(cli: Cli) -> Result<()> {
    check_dependencies(&["yt-dlp", "aria2c", "ffmpeg"])?;

    let destination = cli.destination.as_deref();
    let cookies = cli.cookies_from.as_deref();

    if cli.urls.len() == 1 {
        let url = cli.urls[0].trim();
        if !validate_url(url) {
            return Err(YtrsError::NoValidUrls);
        }

        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?
            .block_on(download_single(url, destination, cookies, cli.socm))
    } else {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?
            .block_on(download_batch(
                cli.urls,
                destination,
                cookies,
                cli.socm,
                cli.parallel,
            ))
    }
}

fn main() {
    let cli = Cli::parse();

    if let Err(e) = run(cli) {
        eprintln!("{} {}", "Error:".red(), e);
        std::process::exit(1);
    }
}
