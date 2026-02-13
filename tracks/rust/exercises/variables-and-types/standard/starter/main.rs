// Log Filter — Variables and Types Exercise (Rust)
//
// Implement a log entry type and filter that demonstrates Rust's ownership
// model, enums, pattern matching, and String vs &str.
//
// Run tests: rustc --test main.rs && ./main

// ---------- Types ----------

#[derive(PartialEq, PartialOrd, Clone, Debug)]
enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

struct LogEntry {
    level: LogLevel,
    message: String,
    source: String,
    timestamp: u64,
    metadata: Option<String>,
}

// ---------- Implementation ----------

impl LogEntry {
    /// Creates a new LogEntry. Takes borrowed strings and owns the data.
    fn new(level: LogLevel, message: &str, source: &str, timestamp: u64) -> LogEntry {
        todo!()
    }

    /// Builder method: attaches metadata. Consumes self and returns a new LogEntry.
    fn with_metadata(self, metadata: &str) -> LogEntry {
        todo!()
    }

    /// Returns true if this entry's level is >= min_level.
    fn matches_level(&self, min_level: &LogLevel) -> bool {
        todo!()
    }

    /// Returns true if keyword appears in the message or metadata.
    fn contains(&self, keyword: &str) -> bool {
        todo!()
    }
}

/// Filters log entries by minimum level. Borrows the slice, returns references.
fn filter_logs<'a>(logs: &'a [LogEntry], min_level: &LogLevel) -> Vec<&'a LogEntry> {
    todo!()
}

/// Returns a summary string with counts per level.
/// Format: "Debug: N, Info: N, Warn: N, Error: N"
fn summarize(logs: &[LogEntry]) -> String {
    todo!()
}

/// Parses a string into a LogLevel (case-insensitive).
/// Returns Err with a descriptive message for invalid input.
fn parse_level(s: &str) -> Result<LogLevel, String> {
    todo!()
}

// ---------- Tests ----------

#[cfg(test)]
mod tests {
    use super::*;

    // -- Construction and ownership --

    #[test]
    fn test_new_entry_owns_strings() {
        let msg = String::from("connection established");
        let src = String::from("tcp-listener");
        let entry = LogEntry::new(LogLevel::Info, &msg, &src, 1000);

        // Original strings are still usable — new() borrowed them
        assert_eq!(msg, "connection established");
        assert_eq!(src, "tcp-listener");

        // Entry owns its own copies
        assert_eq!(entry.message, "connection established");
        assert_eq!(entry.source, "tcp-listener");
        assert_eq!(entry.timestamp, 1000);
        assert_eq!(entry.level, LogLevel::Info);
        assert!(entry.metadata.is_none());
    }

    #[test]
    fn test_new_from_str_literals() {
        let entry = LogEntry::new(LogLevel::Debug, "starting up", "main", 0);
        assert_eq!(entry.message, "starting up");
        assert_eq!(entry.source, "main");
    }

    // -- Builder pattern --

    #[test]
    fn test_with_metadata() {
        let entry = LogEntry::new(LogLevel::Warn, "high latency", "api-gw", 2000)
            .with_metadata("p99=1200ms");

        assert_eq!(entry.metadata, Some(String::from("p99=1200ms")));
        assert_eq!(entry.message, "high latency"); // other fields preserved
        assert_eq!(entry.level, LogLevel::Warn);
    }

    #[test]
    fn test_chained_builder() {
        // with_metadata consumes self, so you chain it directly
        let entry = LogEntry::new(LogLevel::Error, "disk full", "storage", 3000)
            .with_metadata("mount=/var/log");

        assert_eq!(entry.level, LogLevel::Error);
        assert_eq!(entry.metadata, Some(String::from("mount=/var/log")));
    }

    // -- Level comparison --

    #[test]
    fn test_level_ordering() {
        assert!(LogLevel::Debug < LogLevel::Info);
        assert!(LogLevel::Info < LogLevel::Warn);
        assert!(LogLevel::Warn < LogLevel::Error);
        assert!(LogLevel::Debug < LogLevel::Error);
    }

    #[test]
    fn test_matches_level() {
        let entry = LogEntry::new(LogLevel::Warn, "timeout", "http", 100);

        assert!(entry.matches_level(&LogLevel::Debug));  // Warn >= Debug
        assert!(entry.matches_level(&LogLevel::Info));    // Warn >= Info
        assert!(entry.matches_level(&LogLevel::Warn));    // Warn >= Warn
        assert!(!entry.matches_level(&LogLevel::Error));  // Warn < Error
    }

    // -- Keyword search --

    #[test]
    fn test_contains_in_message() {
        let entry = LogEntry::new(LogLevel::Error, "connection refused", "tcp", 100);

        assert!(entry.contains("refused"));
        assert!(entry.contains("connection"));
        assert!(!entry.contains("timeout"));
    }

    #[test]
    fn test_contains_in_metadata() {
        let entry = LogEntry::new(LogLevel::Info, "request complete", "api", 100)
            .with_metadata("status=200 user_id=42");

        assert!(entry.contains("user_id=42"));   // found in metadata
        assert!(entry.contains("request"));        // found in message
        assert!(!entry.contains("error"));         // not found anywhere
    }

    #[test]
    fn test_contains_no_metadata() {
        let entry = LogEntry::new(LogLevel::Debug, "tracing span", "tracer", 100);

        assert!(entry.contains("span"));
        assert!(!entry.contains("missing"));
    }

    // -- Filtering --

    #[test]
    fn test_filter_logs() {
        let logs = vec![
            LogEntry::new(LogLevel::Debug, "trace data", "core", 1),
            LogEntry::new(LogLevel::Info, "started", "main", 2),
            LogEntry::new(LogLevel::Warn, "deprecation", "api", 3),
            LogEntry::new(LogLevel::Error, "crash", "worker", 4),
            LogEntry::new(LogLevel::Info, "healthy", "health", 5),
        ];

        let warnings = filter_logs(&logs, &LogLevel::Warn);
        assert_eq!(warnings.len(), 2);
        assert_eq!(warnings[0].message, "deprecation");
        assert_eq!(warnings[1].message, "crash");

        let all = filter_logs(&logs, &LogLevel::Debug);
        assert_eq!(all.len(), 5);

        let errors = filter_logs(&logs, &LogLevel::Error);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].message, "crash");
    }

    #[test]
    fn test_filter_empty_logs() {
        let logs: Vec<LogEntry> = vec![];
        let result = filter_logs(&logs, &LogLevel::Debug);
        assert!(result.is_empty());
    }

    // -- Summary --

    #[test]
    fn test_summarize() {
        let logs = vec![
            LogEntry::new(LogLevel::Debug, "a", "s", 1),
            LogEntry::new(LogLevel::Debug, "b", "s", 2),
            LogEntry::new(LogLevel::Info, "c", "s", 3),
            LogEntry::new(LogLevel::Error, "d", "s", 4),
        ];

        let summary = summarize(&logs);
        assert_eq!(summary, "Debug: 2, Info: 1, Warn: 0, Error: 1");
    }

    #[test]
    fn test_summarize_empty() {
        let logs: Vec<LogEntry> = vec![];
        let summary = summarize(&logs);
        assert_eq!(summary, "Debug: 0, Info: 0, Warn: 0, Error: 0");
    }

    // -- Parsing --

    #[test]
    fn test_parse_level_valid() {
        assert_eq!(parse_level("debug").unwrap(), LogLevel::Debug);
        assert_eq!(parse_level("INFO").unwrap(), LogLevel::Info);
        assert_eq!(parse_level("Warn").unwrap(), LogLevel::Warn);
        assert_eq!(parse_level("ERROR").unwrap(), LogLevel::Error);
    }

    #[test]
    fn test_parse_level_invalid() {
        let result = parse_level("critical");
        assert!(result.is_err());

        let err_msg = result.unwrap_err();
        assert!(
            err_msg.contains("critical"),
            "Error message should contain the invalid input, got: {}",
            err_msg
        );
    }

    #[test]
    fn test_parse_level_empty() {
        assert!(parse_level("").is_err());
    }
}

fn main() {
    // Example usage — not required for tests
    let logs = vec![
        LogEntry::new(LogLevel::Info, "server started on :8080", "main", 1000),
        LogEntry::new(LogLevel::Debug, "loading config", "init", 999),
        LogEntry::new(LogLevel::Error, "connection refused", "db", 1001)
            .with_metadata("host=10.0.0.5 port=5432"),
        LogEntry::new(LogLevel::Warn, "deprecated endpoint hit", "api", 1002),
    ];

    println!("All logs:");
    for log in &logs {
        println!("  [{:?}] {} (from: {})", log.level, log.message, log.source);
    }

    let warnings = filter_logs(&logs, &LogLevel::Warn);
    println!("\nWarnings and above ({}):", warnings.len());
    for log in &warnings {
        println!("  [{:?}] {}", log.level, log.message);
    }

    println!("\nSummary: {}", summarize(&logs));

    match parse_level("error") {
        Ok(level) => println!("\nParsed level: {:?}", level),
        Err(e) => println!("\nParse error: {}", e),
    }
}
