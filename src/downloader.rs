//! Download orchestration with async execution and concurrency control

use std::num::NonZeroUsize;
use std::path::Path;
use std::process::Stdio;
use std::sync::Arc;

use crate::args_builder::{YtDlpArgs, build_ytdlp_args};
use crate::config::BATCH_SLEEP_THRESHOLD;
use crate::error::{Result, YtrsError, extract_error_reason};
use crate::mode::DownloadMode;
use crate::url_validator::sanitize_and_deduplicate;
use colored::Colorize;
use futures::StreamExt;
use signal_hook::consts::{SIGINT, SIGTERM};
use signal_hook_tokio::Signals;
use tokio::io::AsyncReadExt;
use tokio::process::Command;
use tokio::sync::{Mutex, Semaphore};
use tokio::task::JoinSet;

pub async fn download_single(
    url: &str,
    destination_path: Option<&Path>,
    cookies_from: Option<&str>,
    mode: DownloadMode,
) -> Result<()> {
    let args = YtDlpArgs {
        destination_path,
        cookies_from,
        mode,
        apply_rate_limit: false,
    };

    let cmd_args = build_ytdlp_args(url, &args);
    let cmd_args_str: Vec<String> = cmd_args
        .iter()
        .map(std::string::ToString::to_string)
        .collect();

    let mut child = Command::new("yt-dlp")
        .args(&cmd_args_str)
        .stdout(Stdio::inherit())
        .stderr(Stdio::piped())
        .spawn()?;

    let exit_status = child.wait().await?;

    if !exit_status.success() {
        // Read stderr for error context
        let mut stderr_output = String::new();
        if let Some(mut stderr) = child.stderr.take() {
            let _ = stderr.read_to_string(&mut stderr_output).await;
        }

        let reason = extract_error_reason(&stderr_output, exit_status.code());
        return Err(YtrsError::DownloadFailed {
            url: url.to_string(),
            reason,
        });
    }

    Ok(())
}

struct DownloadContext {
    destination_path: Option<Arc<Path>>,
    cookies_from: Option<Arc<str>>,
    mode: DownloadMode,
    apply_rate_limit: bool,
}

struct FailedDownload {
    url: String,
    reason: String,
}

async fn download_url_task(
    url: String,
    ctx: Arc<DownloadContext>,
    failed_downloads: Arc<Mutex<Vec<FailedDownload>>>,
) {
    println!("{} {}", "Starting:".cyan(), url.cyan());

    let args = YtDlpArgs {
        destination_path: ctx.destination_path.as_deref(),
        cookies_from: ctx.cookies_from.as_deref(),
        mode: ctx.mode,
        apply_rate_limit: ctx.apply_rate_limit,
    };

    let cmd_args = build_ytdlp_args(&url, &args);
    let cmd_args_str: Vec<String> = cmd_args
        .iter()
        .map(std::string::ToString::to_string)
        .collect();

    let result = Command::new("yt-dlp")
        .args(&cmd_args_str)
        .stdout(Stdio::inherit())
        .stderr(Stdio::piped())
        .spawn();

    match result {
        Ok(mut child) => {
            let exit_status = child.wait().await;

            match exit_status {
                Ok(status) if status.success() => {
                    println!("{} {}", "Completed:".green(), url.green());
                }
                Ok(status) => {
                    let mut stderr_output = String::new();
                    if let Some(mut stderr) = child.stderr.take() {
                        let _ = stderr.read_to_string(&mut stderr_output).await;
                    }

                    let reason = extract_error_reason(&stderr_output, status.code());
                    eprintln!("{} {} - {}", "Failed:".red(), url.red(), reason.red());

                    failed_downloads
                        .lock()
                        .await
                        .push(FailedDownload { url, reason });
                }
                Err(e) => {
                    let reason = format!("Process error: {e}");
                    eprintln!("{} {} - {}", "Failed:".red(), url.red(), reason.red());

                    failed_downloads
                        .lock()
                        .await
                        .push(FailedDownload { url, reason });
                }
            }
        }
        Err(e) => {
            let reason = format!("Failed to spawn yt-dlp: {e}");
            eprintln!("{} {} - {}", "Failed:".red(), url.red(), reason.red());

            failed_downloads
                .lock()
                .await
                .push(FailedDownload { url, reason });
        }
    }
}

#[allow(clippy::significant_drop_tightening)]
pub async fn download_batch(
    urls: Vec<String>,
    destination_path: Option<&Path>,
    cookies_from: Option<&str>,
    mode: DownloadMode,
    parallel: NonZeroUsize,
) -> Result<()> {
    let original_count = urls.len();
    let clean_urls = sanitize_and_deduplicate(urls);

    if clean_urls.is_empty() {
        return Err(YtrsError::NoValidUrls);
    }

    let url_count = clean_urls.len();

    if url_count != original_count {
        println!(
            "Processing {} valid URLs (filtered from {})",
            url_count.to_string().cyan(),
            original_count.to_string().cyan()
        );
    }

    let apply_rate_limit = url_count > BATCH_SLEEP_THRESHOLD;
    if apply_rate_limit {
        println!(
            "{} Large batch detected (>{} URLs). Adding sleep intervals to prevent rate limiting.",
            "Note:".yellow(),
            BATCH_SLEEP_THRESHOLD
        );
    }

    let ctx = Arc::new(DownloadContext {
        destination_path: destination_path.map(Arc::from),
        cookies_from: cookies_from.map(Arc::from),
        mode,
        apply_rate_limit,
    });

    let semaphore = Arc::new(Semaphore::new(parallel.get()));
    let failed_downloads = Arc::new(Mutex::new(Vec::new()));
    let mut join_set = JoinSet::new();

    let signals = Signals::new([SIGINT, SIGTERM])?;
    let signals_handle = signals.handle();
    let mut signals_stream = signals.fuse();

    let download_future = async {
        for url in clean_urls {
            let permit = semaphore
                .clone()
                .acquire_owned()
                .await
                .map_err(|_| YtrsError::SemaphoreClosed)?;

            let ctx_clone = ctx.clone();
            let failed_downloads_clone = failed_downloads.clone();

            join_set.spawn(async move {
                download_url_task(url, ctx_clone, failed_downloads_clone).await;
                drop(permit);
            });
        }

        // Wait for all tasks to complete
        while join_set.join_next().await.is_some() {}
        Ok::<(), YtrsError>(())
    };

    // Race between downloads and signals
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

    let failed = failed_downloads.lock().await;
    if !failed.is_empty() {
        println!("\n{}", "─".repeat(50));
        println!("{}", "DOWNLOAD SUMMARY".bold());
        println!("{}", "─".repeat(50));

        println!(
            "{} {}/{} downloads failed",
            "Error:".red().bold(),
            failed.len().to_string().red(),
            url_count.to_string().white()
        );

        println!("\n{}", "Failed downloads:".red().bold());
        for fail in failed.iter() {
            println!("  {} {}", "•".red(), fail.url.red());
            println!("    {} {}", "Reason:".dimmed(), fail.reason.dimmed());
        }

        return Err(YtrsError::PartialFailure(failed.len()));
    }

    println!("\n{}", "─".repeat(50));
    println!("{}", "DOWNLOAD SUMMARY".bold());
    println!("{}", "─".repeat(50));
    println!(
        "{} All {} downloads completed successfully.",
        "Success:".green().bold(),
        url_count
    );

    Ok(())
}
