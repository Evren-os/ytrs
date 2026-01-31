//! Download mode definitions and social media platform presets
//!
//! This module defines the core DownloadMode enum and provides
//! platform-specific configuration for social media optimization

use crate::cli::SocialMediaTarget;

/// Download mode determines format selection, codec priority, and post-processing
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DownloadMode {
    /// Best video+audio, maximum quality (VP9 > AV1 > H.264)
    Default,

    /// Optimized for specific social media platform
    SocialMedia(SocialMediaTarget),

    /// Audio only
    AudioOnly,

    /// Video only
    VideoOnly,
}

impl std::fmt::Display for DownloadMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DownloadMode::Default => write!(f, "Default (Max Quality)"),
            DownloadMode::SocialMedia(target) => write!(f, "Social Media ({})", target),
            DownloadMode::AudioOnly => write!(f, "Audio Only"),
            DownloadMode::VideoOnly => write!(f, "Video Only"),
        }
    }
}

/// Configuration preset for social media platforms
///
/// Each preset defines the optimal encoding parameters for a specific platform,
/// balancing quality with file size and codec compatibility constraints
#[derive(Clone, Debug)]
pub struct SocialMediaPreset {
    /// Maximum file size in MB
    pub max_size_mb: u32,

    /// Maximum resolution height
    pub max_height: u32,

    /// FFmpeg video codec
    pub video_codec: &'static str,

    /// FFmpeg audio codec
    pub audio_codec: &'static str,

    /// Audio bitrate
    pub audio_bitrate: &'static str,

    /// CRF value for encoding
    pub crf: u8,

    /// FFmpeg preset
    pub preset: &'static str,
}

impl SocialMediaTarget {
    /// Get the encoding preset for this platform
    pub fn preset(&self) -> SocialMediaPreset {
        match self {
            // WhatsApp
            SocialMediaTarget::WhatsApp => SocialMediaPreset {
                max_size_mb: 16,
                max_height: 1080,
                video_codec: "libx264",
                audio_codec: "aac",
                audio_bitrate: "128k",
                crf: 23,
                preset: "medium",
            },

            // Discord
            SocialMediaTarget::Discord => SocialMediaPreset {
                max_size_mb: 25,
                max_height: 1080,
                video_codec: "libx264",
                audio_codec: "aac",
                audio_bitrate: "160k",
                crf: 20,
                preset: "medium",
            },

            // Instagram
            SocialMediaTarget::Instagram => SocialMediaPreset {
                max_size_mb: 15,
                max_height: 720,
                video_codec: "libx264",
                audio_codec: "aac",
                audio_bitrate: "128k",
                crf: 23,
                preset: "medium",
            },

            // Messenger
            SocialMediaTarget::Messenger => SocialMediaPreset {
                max_size_mb: 25,
                max_height: 1080,
                video_codec: "libx264",
                audio_codec: "aac",
                audio_bitrate: "160k",
                crf: 20,
                preset: "medium",
            },

            // Signal
            SocialMediaTarget::Signal => SocialMediaPreset {
                max_size_mb: 100,
                max_height: 1080,
                video_codec: "libx264",
                audio_codec: "aac",
                audio_bitrate: "192k",
                crf: 18,
                preset: "slow",
            },

            // Telegram
            SocialMediaTarget::Telegram => SocialMediaPreset {
                max_size_mb: 2000,
                max_height: 2160,
                video_codec: "libx264",
                audio_codec: "aac",
                audio_bitrate: "192k",
                crf: 18,
                preset: "slow",
            },
        }
    }

    pub fn format_selector(&self) -> String {
        let preset = self.preset();
        format!(
            "bv*[height<={}]+ba/b[height<={}]",
            preset.max_height, preset.max_height
        )
    }

    pub fn format_sort(&self) -> String {
        let preset = self.preset();
        format!("res:{},vcodec:avc,acodec:aac,size", preset.max_height)
    }

    pub fn postprocessor_args(&self) -> String {
        let preset = self.preset();
        format!(
            "ffmpeg:-c:v {} -preset {} -crf {} -c:a {} -b:a {} -movflags +faststart",
            preset.video_codec, preset.preset, preset.crf, preset.audio_codec, preset.audio_bitrate,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_download_mode_display() {
        assert_eq!(DownloadMode::Default.to_string(), "Default (Max Quality)");
        assert_eq!(DownloadMode::AudioOnly.to_string(), "Audio Only");
        assert_eq!(DownloadMode::VideoOnly.to_string(), "Video Only");
        assert_eq!(
            DownloadMode::SocialMedia(SocialMediaTarget::Discord).to_string(),
            "Social Media (Discord)"
        );
    }

    #[test]
    fn test_whatsapp_preset() {
        let preset = SocialMediaTarget::WhatsApp.preset();
        assert_eq!(preset.max_size_mb, 16);
        assert_eq!(preset.max_height, 1080);
        assert_eq!(preset.crf, 23); // Higher CRF for smaller files
    }

    #[test]
    fn test_signal_preset() {
        let preset = SocialMediaTarget::Signal.preset();
        assert_eq!(preset.max_size_mb, 100);
        assert_eq!(preset.crf, 18); // Lower CRF for better quality
        assert_eq!(preset.preset, "slow"); // Better compression
    }

    #[test]
    fn test_telegram_preset() {
        let preset = SocialMediaTarget::Telegram.preset();
        assert_eq!(preset.max_size_mb, 2000); // 2GB limit
        assert_eq!(preset.max_height, 2160); // 4K support
        assert_eq!(preset.crf, 18); // High quality
        assert_eq!(preset.preset, "slow");
    }

    #[test]
    fn test_instagram_720p() {
        let preset = SocialMediaTarget::Instagram.preset();
        assert_eq!(preset.max_height, 720); // Instagram optimized
    }

    #[test]
    fn test_format_selector() {
        let selector = SocialMediaTarget::Instagram.format_selector();
        assert_eq!(selector, "bv*[height<=720]+ba/b[height<=720]");
    }

    #[test]
    fn test_format_sort() {
        let sort = SocialMediaTarget::Discord.format_sort();
        assert_eq!(sort, "res:1080,vcodec:avc,acodec:aac,size");
    }

    #[test]
    fn test_postprocessor_args() {
        let args = SocialMediaTarget::WhatsApp.postprocessor_args();
        assert!(args.contains("-crf 23"));
        assert!(args.contains("-b:a 128k"));
        assert!(args.contains("+faststart"));
    }
}