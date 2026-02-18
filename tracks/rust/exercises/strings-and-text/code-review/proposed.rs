// string_utils — Proposed Code for Review
//
// String utilities for normalizing, extracting, and summarizing text fields
// from API responses and log pipelines.
//
// Run: rustc proposed.rs && ./proposed

use std::collections::HashMap;
use std::fmt;

/// A summary of a text field: its truncated preview and word count.
#[derive(Debug)]
pub struct FieldSummary {
    pub preview: String,
    pub word_count: usize,
}

/// Display implementation for FieldSummary.
/// Uses Debug formatting for the preview so it shows with quotes.
impl fmt::Display for FieldSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} ({} words)", self.preview, self.word_count)   // BUG: uses Debug for preview
    }
}

/// Truncates a string to at most `max_chars` Unicode characters.
/// If truncated, appends "...". Returns an owned String.
pub fn truncate(s: &str, max_chars: usize) -> String {
    let char_count = s.chars().count();
    if char_count <= max_chars {
        s.to_string()                          // BUG: allocates even when no truncation needed
    } else {
        let truncated: String = s.chars().take(max_chars).collect();
        truncated + "..."
    }
}

/// Returns a summary of the given text field.
pub fn summarize(text: &str, preview_chars: usize) -> FieldSummary {
    FieldSummary {
        preview: truncate(text, preview_chars),
        word_count: text.split_whitespace().count(),
    }
}

/// Normalizes a service name: trims whitespace, lowercases.
/// Returns an owned String so the caller always has a fresh copy.
pub fn normalize_service_name(name: String) -> String {  // BUG: takes String by value (forces allocation)
    name.trim().to_lowercase()
}

/// Extracts the first n bytes from a UTF-8 string field for use as a short ID.
/// Returns an empty string if the input is shorter than n bytes.
pub fn short_id(s: &str, n: usize) -> String {
    if s.len() < n {
        return String::new();
    }
    s[..n].to_string()   // BUG: byte-slicing without checking char boundary — may panic on non-ASCII
}

/// Builds an index from field names to their normalized values.
/// Returns a HashMap<String, String>.
pub fn build_field_index(fields: &[(&str, &str)]) -> HashMap<String, String> {
    let mut index = HashMap::new();
    for (name, value) in fields {
        // Insert a normalized copy of each value
        index.insert(
            name.to_string(),
            value.trim().to_string(),   // BUG: always allocates even for already-trimmed values
        );
    }
    index
}

/// Checks if a string looks like a valid log level (DEBUG, INFO, WARN, ERROR, TRACE).
/// Case-insensitive.
pub fn is_valid_level(s: &str) -> bool {
    let upper = s.to_uppercase();
    upper == "DEBUG" || upper == "INFO" || upper == "WARN"
        || upper == "ERROR" || upper == "TRACE"
}

/// Returns all values in a HashMap that start with the given prefix.
/// Clones the matching values into a new Vec.
pub fn values_with_prefix(map: &HashMap<String, String>, prefix: &str) -> Vec<String> {
    map.values()
       .filter(|v| v.starts_with(prefix))
       .cloned()    // BUG: clones every matching value — callers may only need &str
       .collect()
}

fn main() {
    let text = "The quick brown fox jumps over the lazy dog";
    let summary = summarize(text, 15);
    println!("summary: {}", summary);

    let name = normalize_service_name("  Auth-Service  ".to_string());
    println!("service: {}", name);

    let id = short_id("request-12345", 7);
    println!("short id: {}", id);

    let fields = vec![
        ("user_id",   "  user-001  "),
        ("action",    "login       "),
        ("status",    "ok"),
    ];
    let index = build_field_index(&fields);
    println!("index: {:?}", index);

    println!("is_valid_level(\"warn\"): {}", is_valid_level("warn"));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_short() {
        assert_eq!(truncate("hello", 10), "hello");
    }

    #[test]
    fn test_truncate_long() {
        assert_eq!(truncate("hello world", 5), "hello...");
    }

    #[test]
    fn test_truncate_unicode() {
        let s = "café au lait";
        let t = truncate(s, 4);
        assert_eq!(t, "café...");
    }

    #[test]
    fn test_normalize_service_name() {
        assert_eq!(normalize_service_name("  Auth-Service  ".to_string()), "  auth-service  ".trim());
    }

    #[test]
    fn test_short_id_ascii() {
        assert_eq!(short_id("request-12345", 7), "request");
    }

    #[test]
    fn test_short_id_too_short() {
        assert_eq!(short_id("abc", 7), "");
    }

    #[test]
    fn test_is_valid_level() {
        assert!(is_valid_level("debug"));
        assert!(is_valid_level("INFO"));
        assert!(is_valid_level("Warn"));
        assert!(!is_valid_level("CRITICAL"));
    }

    #[test]
    fn test_values_with_prefix() {
        let mut map = HashMap::new();
        map.insert("a".to_string(), "prefix_one".to_string());
        map.insert("b".to_string(), "prefix_two".to_string());
        map.insert("c".to_string(), "other".to_string());
        let mut v = values_with_prefix(&map, "prefix_");
        v.sort();
        assert_eq!(v, vec!["prefix_one".to_string(), "prefix_two".to_string()]);
    }
}
