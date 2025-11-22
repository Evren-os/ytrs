mod args_builder;
mod dependencies;
mod downloader;
mod url_validator;

use crate::dependencies::check_dependencies;
use crate::downloader::{download_batch, download_single};
use crate::url_validator::validate_url;
use anyhow::Result;
use clap::Parser;
use colored::Colorize;

#[derive(Parser)]
#[command(
    name = "ytrs",
    about = "A wrapper for yt-dlp to download single videos or batches with optimized settings.",
    long_about = "Automatically detects batch mode when multiple URLs are provided."
)]
struct Cli {
    #[arg(
        short = 'c',
        long,
        default_value = "av1",
        help = "Preferred video codec (av1 or vp9). Ignored if -socm is used."
    )]
    codec: String,

    #[arg(
        short = 'd',
        long,
        help = "Download destination. Can be a directory or a full file path."
    )]
    destination: Option<String>,

    #[arg(
        long,
        help = "Load cookies from the specified browser (e.g., firefox, chrome)."
    )]
    cookies_from: Option<String>,

    #[arg(
        long,
        help = "Optimize for social media compatibility (MP4, H.264/AAC)."
    )]
    socm: bool,

    #[arg(
        short = 'p',
        long,
        default_value = "2",
        help = "Number of parallel downloads for batch mode."
    )]
    parallel: usize,

    #[arg(required = true, help = "URL(s) to download")]
    urls: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.parallel < 1 {
        eprintln!(
            "{} number of parallel downloads (-p) must be at least 1",
            "Error:".red()
        );
        std::process::exit(1);
    }

    check_dependencies(&["yt-dlp", "aria2c"])?;

    if cli.urls.len() == 1 {
        let url = cli.urls[0].trim();
        if !validate_url(url) {
            eprintln!("{} invalid URL provided: {}", "Error:".red(), url);
            std::process::exit(1);
        }

        download_single(url, &cli.codec, cli.destination, cli.cookies_from, cli.socm).await?;
    } else {
        download_batch(
            cli.urls,
            cli.codec,
            cli.destination,
            cli.cookies_from,
            cli.socm,
            cli.parallel,
        )
        .await?;
    }

    Ok(())
}
