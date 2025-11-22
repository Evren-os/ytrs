use colored::Colorize;
use url::Url;

pub fn validate_url(raw_url: &str) -> bool {
    let trimmed = raw_url.trim();
    if trimmed.is_empty() {
        return false;
    }

    match Url::parse(trimmed) {
        Ok(parsed) => {
            let scheme = parsed.scheme();
            scheme == "http" || scheme == "https"
        }
        Err(_) => false,
    }
}

pub fn sanitize_and_deduplicate(urls: Vec<String>) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    let mut result = Vec::new();

    for raw_url in urls {
        let clean = raw_url.trim().to_string();
        if clean.is_empty() {
            continue;
        }

        if !validate_url(&clean) {
            eprintln!(
                "{} {}",
                "Warning: Skipping invalid URL:".yellow(),
                clean.yellow()
            );
            continue;
        }

        if !seen.contains(&clean) {
            seen.insert(clean.clone());
            result.push(clean);
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
            "".to_string(),
            "invalid".to_string(),
        ];
        let result = sanitize_and_deduplicate(urls);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], "https://example.com");
        assert_eq!(result[1], "https://test.com");
    }
}
