use std::process::Command;

use crate::config::MIN_YTDLP_VERSION;
use crate::error::{Result, YtrsError};

pub fn check_dependencies(cmds: &[&str]) -> Result<()> {
    for cmd in cmds {
        if which::which(cmd).is_err() {
            return Err(YtrsError::MissingDependency((*cmd).to_string()));
        }
    }
    Ok(())
}

pub fn enforce_ytdlp_version() -> Result<()> {
    let installed = match Command::new("yt-dlp")
        .args(["--ignore-config", "--version"])
        .output()
    {
        Ok(output) if output.status.success() => String::from_utf8_lossy(&output.stdout)
            .split_whitespace()
            .next()
            .unwrap_or("")
            .to_string(),
        _ => String::new(),
    };
    verify_ytdlp_version(&installed)
}

fn verify_ytdlp_version(installed: &str) -> Result<()> {
    if is_ytdlp_version_supported(installed) {
        return Ok(());
    }
    Err(YtrsError::UnsupportedYtdlpVersion {
        installed: if installed.is_empty() {
            "(unknown)".to_string()
        } else {
            installed.to_string()
        },
        minimum: MIN_YTDLP_VERSION.to_string(),
    })
}

pub fn is_ytdlp_version_supported(version: &str) -> bool {
    let Some(mut installed) = parse_version(version) else {
        return false;
    };
    let Some(mut minimum) = parse_version(MIN_YTDLP_VERSION) else {
        return false;
    };
    let len = installed.len().max(minimum.len());
    installed.resize(len, 0);
    minimum.resize(len, 0);
    installed >= minimum
}

fn parse_version(version: &str) -> Option<Vec<u64>> {
    let trimmed = version.trim();
    if trimmed.is_empty() {
        return None;
    }
    let mut components = Vec::new();
    for part in trimmed.split('.') {
        if part.is_empty() || !part.bytes().all(|b| b.is_ascii_digit()) {
            return None;
        }
        components.push(part.parse().ok()?);
    }
    Some(components)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_predicate_accepts_minimum() {
        assert!(is_ytdlp_version_supported("2026.06.09"));
    }

    #[test]
    fn test_version_predicate_table() {
        let cases = [
            ("2026.06.09", true, "equal to minimum"),
            ("2026.06.10", true, "newer"),
            ("2027.01.01", true, "newer year"),
            (
                "2026.06.09.230517",
                true,
                "same-day nightly with fourth component",
            ),
            (
                "2026.06.09.000001",
                true,
                "nightly with small fourth component",
            ),
            ("2026.06", false, "missing components treated as zero"),
            ("2026.06.08", false, "older"),
            ("2025.12.31", false, "older year"),
            ("2021.12.17", false, "youtube-dl-style version"),
            ("garbage", false, "non-numeric"),
            ("2026.06.09.x", false, "partially non-numeric"),
            ("2026..09", false, "empty component"),
            ("", false, "empty output"),
            ("   ", false, "whitespace-only output"),
        ];
        for (input, expected, label) in cases {
            assert_eq!(
                is_ytdlp_version_supported(input),
                expected,
                "{label}: {input:?}"
            );
        }
    }

    #[test]
    fn test_version_gate_accepts_patched_install() {
        assert!(verify_ytdlp_version("2026.06.09\n").is_ok());
        assert!(verify_ytdlp_version("2026.06.09.230517\n").is_ok());
        assert!(verify_ytdlp_version("2027.01.01\n").is_ok());
    }

    #[test]
    fn test_version_gate_rejects_old_install() {
        let err = verify_ytdlp_version("2025.12.31\n").unwrap_err();
        assert!(
            matches!(err, YtrsError::UnsupportedYtdlpVersion { .. }),
            "old install must yield the version-gate error, got: {err}"
        );
    }

    #[test]
    fn test_version_gate_fails_closed_on_unparseable() {
        for output in ["garbage\n", "\n", ""] {
            let err = verify_ytdlp_version(output).unwrap_err();
            assert!(
                matches!(err, YtrsError::UnsupportedYtdlpVersion { .. }),
                "unparseable output {output:?} must fail closed, got: {err}"
            );
        }
    }

    #[test]
    fn test_version_gate_reports_unknown_for_empty_output() {
        let err = verify_ytdlp_version("").unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("(unknown)"),
            "empty output should name the installed version as unknown: {msg}"
        );
    }

    #[test]
    fn test_check_existing_command() {
        let result = check_dependencies(&["sh"]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_missing_command() {
        let result = check_dependencies(&["nonexistent_command_xyz"]);
        assert!(result.is_err());
    }
}
