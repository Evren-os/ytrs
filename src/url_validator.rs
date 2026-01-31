use std::collections::HashSet;

use colored::Colorize;
use url::Url;

pub fn validate_url(raw_url: &str) -> bool {
    let trimmed = raw_url.trim();
    if trimmed.is_empty() {
        return false;
    }

    Url::parse(trimmed)
        .map(|parsed| matches!(parsed.scheme(), "http" | "https"))
        .unwrap_or(false)
}

pub fn sanitize_and_deduplicate(urls: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::with_capacity(urls.len());
    let mut result = Vec::with_capacity(urls.len());

    for raw_url in urls {
        let trimmed = raw_url.trim();
        if trimmed.is_empty() {
            continue;
        }

        if !validate_url(trimmed) {
            eprintln!(
                "{} {}",
                "Warning: Skipping invalid URL:".yellow(),
                trimmed.yellow()
            );
            continue;
        }

        if seen.insert(trimmed.to_string()) {
            result.push(trimmed.to_string());
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_url() {
        assert!(validate_url("https://www.youtube.com/watch?v=test"));
        assert!(validate_url("http://example.com"));
        assert!(!validate_url(""));
        assert!(!validate_url("not-a-url"));
        assert!(!validate_url("ftp://example.com"));
    }

    #[test]
    fn test_sanitize_and_deduplicate() {
        let urls = vec![
            "https://example.com".to_string(),
            "https://example.com".to_string(),
            "https://test.com".to_string(),
            String::new(),
            "invalid".to_string(),
        ];
        let result = sanitize_and_deduplicate(urls);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], "https://example.com");
        assert_eq!(result[1], "https://test.com");
    }
}
