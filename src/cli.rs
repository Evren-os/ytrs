//! Command-line interface definitions for ytrs
//!
//! This module defines the CLI structure using clap derive macros,
//! including the `SocialMediaTarget` enum for platform-specific presets

use std::num::NonZeroUsize;
use std::path::PathBuf;

use clap::{Parser, ValueEnum};

use crate::error::{Result, YtrsError};
use crate::mode::DownloadMode;

/// Social media platforms
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum SocialMediaTarget {
    /// `WhatsApp`: 16MB limit, H.264/AAC, 1080p
    #[value(name = "whatsapp", alias = "wa")]
    WhatsApp,

    /// Discord: 25MB, broad codec support, 1080p
    #[value(name = "discord", alias = "dc")]
    Discord,

    /// Instagram: 15MB limit, H.264/AAC, 720p
    #[value(name = "instagram", alias = "ig")]
    Instagram,

    /// Facebook: 25MB limit, H.264/AAC, 1080p
    #[value(name = "messenger", alias = "fb")]
    Messenger,

    /// Signal: 100MB limit, H.264/AAC, 1080p
    #[value(name = "signal", alias = "sig")]
    Signal,

    /// Telegram: 2GB limit, H.264/AAC, 4K
    #[value(name = "telegram", alias = "tg")]
    Telegram,
}

impl std::fmt::Display for SocialMediaTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::WhatsApp => write!(f, "WhatsApp"),
            Self::Discord => write!(f, "Discord"),
            Self::Instagram => write!(f, "Instagram"),
            Self::Messenger => write!(f, "Messenger"),
            Self::Signal => write!(f, "Signal"),
            Self::Telegram => write!(f, "Telegram"),
        }
    }
}

#[derive(Parser, Debug)]
#[command(
    name = "ytrs",
    version,
    about = "High-performance yt-dlp wrapper with social media optimization",
    long_about = "Downloads media from yt-dlp supported sites with maximum quality (VP9 > AV1 > H.264).\n\n\
                  Supports batch downloads, audio/video-only modes, and platform-specific \n\
                  social media optimization for WhatsApp, Discord, Instagram, Messenger, and Signal."
)]
pub struct Cli {
    /// Download destination
    #[arg(short = 'd', long, value_name = "PATH")]
    pub destination: Option<PathBuf>,

    /// Load cookies from browser
    #[arg(long, value_name = "BROWSER")]
    pub cookies_from: Option<String>,

    /// Optimize for social media platform
    ///
    /// Tuned settings for each platform (file size, codec, and resolution)
    /// Short aliases: wa, dc, ig, fb, sig
    #[arg(long, value_name = "PLATFORM")]
    pub socm: Option<SocialMediaTarget>,

    /// Download audio only
    #[arg(short = 'a', long = "audio", conflicts_with_all = ["video_only", "socm"])]
    pub audio_only: bool,

    /// Download video only
    #[arg(short = 'v', long = "video", conflicts_with_all = ["audio_only", "socm"])]
    pub video_only: bool,

    /// Number of parallel downloads for batch mode
    #[arg(short = 'p', long, default_value = "2", value_name = "N")]
    pub parallel: NonZeroUsize,

    /// URL(s) to download
    #[arg(required = true, value_name = "URL")]
    pub urls: Vec<String>,
}

impl Cli {
    /// Determine the download mode from CLI arguments
    pub fn download_mode(&self) -> Result<DownloadMode> {
        if self.audio_only && self.video_only {
            return Err(YtrsError::InvalidModeCombo(
                "Cannot use --audio and --video together".to_string(),
            ));
        }

        if self.audio_only && self.socm.is_some() {
            return Err(YtrsError::InvalidModeCombo(
                "Cannot use --audio with --socm".to_string(),
            ));
        }

        if self.video_only && self.socm.is_some() {
            return Err(YtrsError::InvalidModeCombo(
                "Cannot use --video with --socm".to_string(),
            ));
        }

        Ok(match (self.audio_only, self.video_only, &self.socm) {
            (true, false, None) => DownloadMode::AudioOnly,
            (false, true, None) => DownloadMode::VideoOnly,
            (false, false, Some(target)) => DownloadMode::SocialMedia(*target),
            (false, false, None) => DownloadMode::Default,
            _ => unreachable!("Invalid mode combination should be caught by clap"),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_social_media_target_display() {
        assert_eq!(SocialMediaTarget::WhatsApp.to_string(), "WhatsApp");
        assert_eq!(SocialMediaTarget::Discord.to_string(), "Discord");
        assert_eq!(SocialMediaTarget::Instagram.to_string(), "Instagram");
        assert_eq!(SocialMediaTarget::Messenger.to_string(), "Messenger");
        assert_eq!(SocialMediaTarget::Signal.to_string(), "Signal");
    }

    #[test]
    fn test_download_mode_default() {
        let cli = Cli {
            destination: None,
            cookies_from: None,
            socm: None,
            audio_only: false,
            video_only: false,
            parallel: NonZeroUsize::new(2).unwrap(),
            urls: vec!["https://example.com".to_string()],
        };
        assert_eq!(cli.download_mode().unwrap(), DownloadMode::Default);
    }

    #[test]
    fn test_download_mode_audio() {
        let cli = Cli {
            destination: None,
            cookies_from: None,
            socm: None,
            audio_only: true,
            video_only: false,
            parallel: NonZeroUsize::new(2).unwrap(),
            urls: vec!["https://example.com".to_string()],
        };
        assert_eq!(cli.download_mode().unwrap(), DownloadMode::AudioOnly);
    }

    #[test]
    fn test_download_mode_video() {
        let cli = Cli {
            destination: None,
            cookies_from: None,
            socm: None,
            audio_only: false,
            video_only: true,
            parallel: NonZeroUsize::new(2).unwrap(),
            urls: vec!["https://example.com".to_string()],
        };
        assert_eq!(cli.download_mode().unwrap(), DownloadMode::VideoOnly);
    }

    #[test]
    fn test_download_mode_socm() {
        let cli = Cli {
            destination: None,
            cookies_from: None,
            socm: Some(SocialMediaTarget::Discord),
            audio_only: false,
            video_only: false,
            parallel: NonZeroUsize::new(2).unwrap(),
            urls: vec!["https://example.com".to_string()],
        };
        assert!(matches!(
            cli.download_mode().unwrap(),
            DownloadMode::SocialMedia(SocialMediaTarget::Discord)
        ));
    }
}
