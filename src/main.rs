//! ytrs - High-performance yt-dlp wrapper with social media optimization

mod args_builder;
mod cli;
mod config;
mod dependencies;
mod downloader;
mod error;
mod mode;
mod url_validator;

use clap::Parser;
use colored::Colorize;

use crate::cli::Cli;
use crate::config::REQUIRED_DEPENDENCIES;
use crate::dependencies::check_dependencies;
use crate::downloader::{download_batch, download_single};
use crate::error::{Result, YtrsError};
use crate::url_validator::validate_url;

fn run(cli: Cli) -> Result<()> {
    check_dependencies(REQUIRED_DEPENDENCIES)?;

    let mode = cli.download_mode()?;

    println!("{} {}", "Mode:".dimmed(), mode.to_string().cyan());

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
            .block_on(download_single(url, destination, cookies, mode))
    } else {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?
            .block_on(download_batch(
                cli.urls,
                destination,
                cookies,
                mode,
                cli.parallel,
            ))
    }
}

fn main() {
    let cli = Cli::parse();

    if let Err(e) = run(cli) {
        eprintln!("{} {}", "Error:".red().bold(), e);
        std::process::exit(1);
    }
}
