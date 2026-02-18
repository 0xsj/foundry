// config-loader/src/util.rs
//
// Utility functions shared across modules.
// This file exists but is not wired into the module tree — that's one of the bugs.

// BUG 6: parse_duration is used in main.rs as `use util::parse_duration`,
// but the function is private here. It needs to be pub.
fn parse_duration(s: &str) -> Option<u64> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }

    if let Some(secs) = s.strip_suffix('s') {
        secs.parse::<u64>().ok().map(|n| n * 1000)
    } else if let Some(ms) = s.strip_suffix("ms") {
        ms.parse::<u64>().ok()
    } else {
        s.parse::<u64>().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_seconds() {
        assert_eq!(parse_duration("30s"), Some(30_000));
    }

    #[test]
    fn test_parse_millis() {
        assert_eq!(parse_duration("500ms"), Some(500));
    }

    #[test]
    fn test_parse_raw_number() {
        assert_eq!(parse_duration("1000"), Some(1000));
    }

    #[test]
    fn test_parse_empty() {
        assert_eq!(parse_duration(""), None);
    }

    #[test]
    fn test_parse_invalid() {
        assert_eq!(parse_duration("fast"), None);
    }
}
