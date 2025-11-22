use crate::args_builder::{build_ytdlp_args, YtDlpArgs};
use crate::url_validator::sanitize_and_deduplicate;
use anyhow::{anyhow, Result};
use colored::Colorize;
use futures::StreamExt;
use signal_hook::consts::{SIGINT, SIGTERM};
use signal_hook_tokio::Signals;
use std::sync::Arc;
use tokio::process::Command;
use tokio::sync::{Mutex, Semaphore};
use tokio::task::JoinSet;

pub async fn download_single(
    url: &str,
    codec_pref: &str,
    destination_path: Option<String>,
    cookies_from: Option<String>,
    socm: bool,
) -> Result<()> {
    let args = YtDlpArgs {
        codec_pref: codec_pref.to_string(),
        destination_path,
        cookies_from,
        socm,
    };

    let cmd_args = build_ytdlp_args(url, &args);

    let status = Command::new("yt-dlp").args(&cmd_args).status().await?;

    if !status.success() {
        return Err(anyhow!("yt-dlp failed with exit code: {:?}", status.code()));
    }

    Ok(())
}

async fn download_url_task(
    url: String,
    codec_pref: String,
    destination_path: Option<String>,
    cookies_from: Option<String>,
    socm: bool,
    failed_urls: Arc<Mutex<Vec<String>>>,
) {
    println!("{} {}", "Starting download:".cyan(), url.cyan());

    let args = YtDlpArgs {
        codec_pref,
        destination_path,
        cookies_from,
        socm,
    };

    let cmd_args = build_ytdlp_args(&url, &args);

    match Command::new("yt-dlp").args(&cmd_args).status().await {
        Ok(status) if status.success() => {
            println!("{} {}", "Completed download:".green(), url.green());
        }
        Ok(status) => {
            eprintln!(
                "{} {} (exit code: {:?})",
                "Failed to download:".red(),
                url.red(),
                status.code()
            );
            failed_urls.lock().await.push(url);
        }
        Err(e) => {
            eprintln!(
                "{} {} (error: {})",
                "Failed to download:".red(),
                url.red(),
                e
            );
            failed_urls.lock().await.push(url);
        }
    }
}

pub async fn download_batch(
    urls: Vec<String>,
    codec_pref: String,
    destination_path: Option<String>,
    cookies_from: Option<String>,
    socm: bool,
    parallel: usize,
) -> Result<()> {
    let clean_urls = sanitize_and_deduplicate(urls.clone());

    if clean_urls.is_empty() {
        return Err(anyhow!("no valid URLs provided"));
    }

    if clean_urls.len() != urls.len() {
        println!(
            "Processing {} valid URLs (filtered from {})",
            clean_urls.len().to_string().cyan(),
            urls.len().to_string().cyan()
        );
    }

    let semaphore = Arc::new(Semaphore::new(parallel));
    let failed_urls = Arc::new(Mutex::new(Vec::new()));
    let mut join_set = JoinSet::new();

    let signals = Signals::new([SIGINT, SIGTERM])?;
    let signals_handle = signals.handle();
    let mut signals_stream = signals.fuse();

    let download_future = async {
        for url in clean_urls.clone() {
            let permit = semaphore
                .clone()
                .acquire_owned()
                .await
                .expect("semaphore closed unexpectedly");
            let failed_urls_clone = failed_urls.clone();
            let codec_pref_clone = codec_pref.clone();
            let destination_path_clone = destination_path.clone();
            let cookies_from_clone = cookies_from.clone();

            join_set.spawn(async move {
                download_url_task(
                    url,
                    codec_pref_clone,
                    destination_path_clone,
                    cookies_from_clone,
                    socm,
                    failed_urls_clone,
                )
                .await;
                drop(permit);
            });
        }

        while join_set.join_next().await.is_some() {}
    };

    tokio::select! {
        _ = download_future => {},
        signal = signals_stream.next() => {
            if signal.is_some() {
                eprintln!(
                    "\n{} {}",
                    "Received termination signal.".yellow(),
                    "Waiting for active downloads to complete...".yellow()
                );
                join_set.shutdown().await;
            }
        }
    }

    signals_handle.close();

    let failed = failed_urls.lock().await;
    if !failed.is_empty() {
        println!("\n--- Summary ---");
        eprintln!(
            "{} {}/{} downloads failed.",
            "Error:".red(),
            failed.len().to_string().red(),
            clean_urls.len().to_string().red()
        );
        println!("Failed URLs:");
        for url in failed.iter() {
            println!("  - {}", url.red());
        }
        return Err(anyhow!("{} downloads failed", failed.len()));
    } else {
        println!("\n--- Summary ---");
        println!(
            "{} All {} downloads completed successfully.",
            "Success:".green(),
            clean_urls.len()
        );
    }

    Ok(())
}
