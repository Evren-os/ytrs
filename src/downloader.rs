use std::num::NonZeroUsize;
use std::path::Path;
use std::sync::Arc;

use colored::Colorize;
use futures::StreamExt;
use signal_hook::consts::{SIGINT, SIGTERM};
use signal_hook_tokio::Signals;
use tokio::process::Command;
use tokio::sync::{Mutex, Semaphore};
use tokio::task::JoinSet;

use crate::args_builder::{YtDlpArgs, build_ytdlp_args};
use crate::error::{Result, YtrsError};
use crate::url_validator::sanitize_and_deduplicate;

pub async fn download_single(
    url: &str,
    destination_path: Option<&Path>,
    cookies_from: Option<&str>,
    socm: bool,
) -> Result<()> {
    let args = YtDlpArgs {
        destination_path,
        cookies_from,
        socm,
    };

    let cmd_args = build_ytdlp_args(url, &args);
    let status = Command::new("yt-dlp")
        .args(cmd_args.iter().map(AsRef::as_ref))
        .status()
        .await?;

    if !status.success() {
        return Err(YtrsError::YtDlpFailed(status.code()));
    }

    Ok(())
}

struct DownloadContext {
    destination_path: Option<Arc<Path>>,
    cookies_from: Option<Arc<str>>,
    socm: bool,
}

async fn download_url_task(
    url: String,
    ctx: Arc<DownloadContext>,
    failed_urls: Arc<Mutex<Vec<String>>>,
) {
    println!("{} {}", "Starting download:".cyan(), url.cyan());

    let args = YtDlpArgs {
        destination_path: ctx.destination_path.as_deref(),
        cookies_from: ctx.cookies_from.as_deref(),
        socm: ctx.socm,
    };

    let cmd_args = build_ytdlp_args(&url, &args);

    match Command::new("yt-dlp")
        .args(cmd_args.iter().map(AsRef::as_ref))
        .status()
        .await
    {
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
    destination_path: Option<&Path>,
    cookies_from: Option<&str>,
    socm: bool,
    parallel: NonZeroUsize,
) -> Result<()> {
    let original_count = urls.len();
    let clean_urls = sanitize_and_deduplicate(urls);

    if clean_urls.is_empty() {
        return Err(YtrsError::NoValidUrls);
    }

    if clean_urls.len() != original_count {
        println!(
            "Processing {} valid URLs (filtered from {})",
            clean_urls.len().to_string().cyan(),
            original_count.to_string().cyan()
        );
    }

    let ctx = Arc::new(DownloadContext {
        destination_path: destination_path.map(Arc::from),
        cookies_from: cookies_from.map(Arc::from),
        socm,
    });

    let semaphore = Arc::new(Semaphore::new(parallel.get()));
    let failed_urls = Arc::new(Mutex::new(Vec::new()));
    let mut join_set = JoinSet::new();

    let signals = Signals::new([SIGINT, SIGTERM])?;
    let signals_handle = signals.handle();
    let mut signals_stream = signals.fuse();

    let total_urls = clean_urls.len();

    let download_future = async {
        for url in clean_urls {
            let permit = semaphore
                .clone()
                .acquire_owned()
                .await
                .map_err(|_| YtrsError::SemaphoreClosed)?;

            let ctx_clone = ctx.clone();
            let failed_urls_clone = failed_urls.clone();

            join_set.spawn(async move {
                download_url_task(url, ctx_clone, failed_urls_clone).await;
                drop(permit);
            });
        }

        while join_set.join_next().await.is_some() {}
        Ok::<(), YtrsError>(())
    };

    tokio::select! {
        result = download_future => result?,
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
            total_urls.to_string().red()
        );
        println!("Failed URLs:");
        for url in failed.iter() {
            println!("  - {}", url.red());
        }
        return Err(YtrsError::PartialFailure(failed.len()));
    }

    println!("\n--- Summary ---");
    println!(
        "{} All {} downloads completed successfully.",
        "Success:".green(),
        total_urls
    );

    Ok(())
}
