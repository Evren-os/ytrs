//! Download modes and social media presets

use crate::cli::SocialMediaTarget;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DownloadMode {
    #[default]
    Default,
    SocialMedia(SocialMediaTarget),
    AudioOnly,
    VideoOnly,
}

impl std::fmt::Display for DownloadMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Default => write!(f, "Default (Max Quality)"),
            Self::SocialMedia(target) => write!(f, "Social Media ({target})"),
            Self::AudioOnly => write!(f, "Audio Only"),
            Self::VideoOnly => write!(f, "Video Only"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct SocialMediaPreset {
    #[allow(dead_code)]
    pub max_size_mb: u32,
    pub max_height: u32,
    pub video_codec: &'static str,
    pub audio_codec: &'static str,
    pub audio_bitrate: &'static str,
    pub crf: u8,
    pub preset: &'static str,
}

impl SocialMediaTarget {
    #[must_use]
    pub const fn preset(self) -> SocialMediaPreset {
        match self {
            Self::WhatsApp => SocialMediaPreset {
                max_size_mb: 16,
                max_height: 1080,
                video_codec: "libx264",
                audio_codec: "aac",
                audio_bitrate: "128k",
                crf: 23,
                preset: "medium",
            },
            Self::Discord | Self::Messenger => SocialMediaPreset {
                max_size_mb: 25,
                max_height: 1080,
                video_codec: "libx264",
                audio_codec: "aac",
                audio_bitrate: "160k",
                crf: 20,
                preset: "medium",
            },
            Self::Instagram => SocialMediaPreset {
                max_size_mb: 15,
                max_height: 720,
                video_codec: "libx264",
                audio_codec: "aac",
                audio_bitrate: "128k",
                crf: 23,
                preset: "medium",
            },
            Self::Signal => SocialMediaPreset {
                max_size_mb: 100,
                max_height: 1080,
                video_codec: "libx264",
                audio_codec: "aac",
                audio_bitrate: "192k",
                crf: 18,
                preset: "slow",
            },
            Self::Telegram => SocialMediaPreset {
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

    #[must_use]
    pub fn format_selector(self) -> String {
        let preset = self.preset();
        format!(
            "bv*[height<={}]+ba/b[height<={}]",
            preset.max_height, preset.max_height
        )
    }

    #[must_use]
    pub fn format_sort(self) -> String {
        let preset = self.preset();
        format!("res:{},vcodec:avc,acodec:aac,size", preset.max_height)
    }

    #[must_use]
    pub fn postprocessor_args(self) -> String {
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
        assert_eq!(preset.crf, 23);
    }

    #[test]
    fn test_signal_preset() {
        let preset = SocialMediaTarget::Signal.preset();
        assert_eq!(preset.max_size_mb, 100);
        assert_eq!(preset.crf, 18);
        assert_eq!(preset.preset, "slow");
    }

    #[test]
    fn test_telegram_preset() {
        let preset = SocialMediaTarget::Telegram.preset();
        assert_eq!(preset.max_size_mb, 2000);
        assert_eq!(preset.max_height, 2160);
        assert_eq!(preset.crf, 18);
        assert_eq!(preset.preset, "slow");
    }

    #[test]
    fn test_instagram_720p() {
        let preset = SocialMediaTarget::Instagram.preset();
        assert_eq!(preset.max_height, 720);
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
