// Log Analyzer — Collections Exercise (Rust)
//
// Implement a log ingestion and analysis pipeline using Vec, HashMap, and BTreeMap.
//
// Run tests: rustc --test main.rs && ./main

use std::collections::{BTreeMap, HashMap};

// ---------- Types ----------

struct LogEntry {
    level: &'static str,
    source: &'static str,
    message: String,
    timestamp_ms: u64,
}

struct LogAnalyzer {
    entries: Vec<LogEntry>,
    level_counts: HashMap<&'static str, u32>,
    source_counts: HashMap<&'static str, u32>,
}

// ---------- Implementation ----------

impl LogEntry {
    /// Creates a new LogEntry. Takes message as &str and converts to owned String.
    fn new(level: &'static str, source: &'static str, message: &str, timestamp_ms: u64) -> LogEntry {
        todo!()
    }
}

impl LogAnalyzer {
    /// Creates an empty LogAnalyzer with all collections initialized.
    fn new() -> LogAnalyzer {
        todo!()
    }

    /// Ingests a log entry: stores it and updates level and source counts.
    /// Use the entry API — not contains_key + insert.
    fn ingest(&mut self, entry: LogEntry) {
        todo!()
    }

    /// Returns a BTreeMap of level -> count (sorted alphabetically by level).
    /// Levels with zero entries are omitted.
    fn level_summary(&self) -> BTreeMap<&'static str, u32> {
        todo!()
    }

    /// Returns the top n sources by entry count, descending.
    /// Ties broken by source name ascending.
    fn top_sources(&self, n: usize) -> Vec<(&'static str, u32)> {
        todo!()
    }

    /// Returns references to all ERROR entries, in insertion order.
    fn errors(&self) -> Vec<&LogEntry> {
        todo!()
    }

    /// Returns the top n (message, count) pairs from ERROR entries, by count descending.
    /// Ties broken by message alphabetically.
    fn most_frequent_errors(&self, n: usize) -> Vec<(&str, u32)> {
        todo!()
    }

    /// Returns a sorted list of sources that emitted at least one ERROR entry.
    /// No duplicates.
    fn sources_with_errors(&self) -> Vec<&'static str> {
        todo!()
    }

    /// Returns references to all entries where start_ms <= timestamp_ms <= end_ms.
    fn entries_in_range(&self, start_ms: u64, end_ms: u64) -> Vec<&LogEntry> {
        todo!()
    }
}

// ---------- Tests ----------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_analyzer() -> LogAnalyzer {
        let mut a = LogAnalyzer::new();
        a.ingest(LogEntry::new("INFO",  "api",    "request received",          1000));
        a.ingest(LogEntry::new("ERROR", "db",     "connection refused",         1001));
        a.ingest(LogEntry::new("DEBUG", "api",    "query plan selected",        1002));
        a.ingest(LogEntry::new("ERROR", "api",    "connection refused",         1003));
        a.ingest(LogEntry::new("WARN",  "worker", "queue depth 900",            1004));
        a.ingest(LogEntry::new("ERROR", "db",     "connection refused",         1005));
        a.ingest(LogEntry::new("INFO",  "api",    "response sent 200",          1006));
        a.ingest(LogEntry::new("ERROR", "worker", "job failed: timeout",        1007));
        a.ingest(LogEntry::new("INFO",  "worker", "job completed",              1008));
        a.ingest(LogEntry::new("ERROR", "db",     "connection refused",         1009));
        a
    }

    // -- Construction --

    #[test]
    fn test_new_is_empty() {
        let a = LogAnalyzer::new();
        assert_eq!(a.entries.len(), 0);
        assert!(a.level_counts.is_empty());
        assert!(a.source_counts.is_empty());
    }

    // -- Ingest --

    #[test]
    fn test_ingest_adds_entry() {
        let mut a = LogAnalyzer::new();
        a.ingest(LogEntry::new("INFO", "api", "started", 100));

        assert_eq!(a.entries.len(), 1);
        assert_eq!(a.entries[0].level, "INFO");
        assert_eq!(a.entries[0].source, "api");
        assert_eq!(a.entries[0].message, "started");
        assert_eq!(a.entries[0].timestamp_ms, 100);
    }

    #[test]
    fn test_ingest_updates_counts() {
        let a = make_analyzer();

        assert_eq!(a.level_counts["INFO"], 3);
        assert_eq!(a.level_counts["ERROR"], 5);
        assert_eq!(a.level_counts["WARN"], 1);
        assert_eq!(a.level_counts["DEBUG"], 1);

        assert_eq!(a.source_counts["api"], 4);
        assert_eq!(a.source_counts["db"], 3);
        assert_eq!(a.source_counts["worker"], 3);
    }

    // -- level_summary --

    #[test]
    fn test_level_summary_sorted() {
        let a = make_analyzer();
        let summary = a.level_summary();

        // BTreeMap is sorted alphabetically — DEBUG < ERROR < INFO < WARN
        let keys: Vec<&&str> = summary.keys().collect();
        assert_eq!(keys, vec![&"DEBUG", &"ERROR", &"INFO", &"WARN"]);

        assert_eq!(summary["DEBUG"], 1);
        assert_eq!(summary["ERROR"], 5);
        assert_eq!(summary["INFO"], 3);
        assert_eq!(summary["WARN"], 1);
    }

    #[test]
    fn test_level_summary_empty() {
        let a = LogAnalyzer::new();
        assert!(a.level_summary().is_empty());
    }

    // -- top_sources --

    #[test]
    fn test_top_sources_n2() {
        let a = make_analyzer();
        let top = a.top_sources(2);

        assert_eq!(top.len(), 2);
        assert_eq!(top[0], ("api", 4));
        // db and worker both have 3 — tie broken alphabetically: "db" < "worker"
        assert_eq!(top[1], ("db", 3));
    }

    #[test]
    fn test_top_sources_all() {
        let a = make_analyzer();
        let top = a.top_sources(100);  // more than available
        assert_eq!(top.len(), 3);
    }

    #[test]
    fn test_top_sources_tie_breaking() {
        let mut a = LogAnalyzer::new();
        a.ingest(LogEntry::new("INFO", "zebra", "msg", 1));
        a.ingest(LogEntry::new("INFO", "alpha", "msg", 2));
        a.ingest(LogEntry::new("INFO", "mango", "msg", 3));
        // all sources have count=1, so sorted alphabetically
        let top = a.top_sources(3);
        assert_eq!(top[0].0, "alpha");
        assert_eq!(top[1].0, "mango");
        assert_eq!(top[2].0, "zebra");
    }

    // -- errors --

    #[test]
    fn test_errors_returns_error_entries() {
        let a = make_analyzer();
        let errors = a.errors();

        assert_eq!(errors.len(), 5);
        assert!(errors.iter().all(|e| e.level == "ERROR"));
    }

    #[test]
    fn test_errors_insertion_order() {
        let a = make_analyzer();
        let errors = a.errors();

        // First error: "db" at ts=1001
        assert_eq!(errors[0].source, "db");
        assert_eq!(errors[0].timestamp_ms, 1001);

        // Second error: "api" at ts=1003
        assert_eq!(errors[1].source, "api");
    }

    // -- most_frequent_errors --

    #[test]
    fn test_most_frequent_errors_top1() {
        let a = make_analyzer();
        let top = a.most_frequent_errors(1);

        assert_eq!(top.len(), 1);
        assert_eq!(top[0].0, "connection refused");
        assert_eq!(top[0].1, 3);  // appears 3 times across db and api
    }

    #[test]
    fn test_most_frequent_errors_top2() {
        let a = make_analyzer();
        let top = a.most_frequent_errors(2);

        assert_eq!(top.len(), 2);
        assert_eq!(top[0].0, "connection refused");
        assert_eq!(top[0].1, 3);
        assert_eq!(top[1].0, "job failed: timeout");
        assert_eq!(top[1].1, 1);
    }

    // -- sources_with_errors --

    #[test]
    fn test_sources_with_errors() {
        let a = make_analyzer();
        let sources = a.sources_with_errors();

        // api, db, worker all emitted errors — sorted alphabetically
        assert_eq!(sources, vec!["api", "db", "worker"]);
    }

    #[test]
    fn test_sources_with_errors_no_duplicates() {
        let mut a = LogAnalyzer::new();
        a.ingest(LogEntry::new("ERROR", "db", "disk full", 1));
        a.ingest(LogEntry::new("ERROR", "db", "disk full", 2));
        a.ingest(LogEntry::new("ERROR", "db", "disk full", 3));

        let sources = a.sources_with_errors();
        assert_eq!(sources.len(), 1);
        assert_eq!(sources[0], "db");
    }

    #[test]
    fn test_sources_with_errors_empty_when_no_errors() {
        let mut a = LogAnalyzer::new();
        a.ingest(LogEntry::new("INFO", "api", "ok", 1));
        assert!(a.sources_with_errors().is_empty());
    }

    // -- entries_in_range --

    #[test]
    fn test_entries_in_range_inclusive() {
        let a = make_analyzer();
        let range = a.entries_in_range(1001, 1004);

        assert_eq!(range.len(), 4);
        assert_eq!(range[0].timestamp_ms, 1001);
        assert_eq!(range[3].timestamp_ms, 1004);
    }

    #[test]
    fn test_entries_in_range_empty() {
        let a = make_analyzer();
        let range = a.entries_in_range(9000, 9999);
        assert!(range.is_empty());
    }

    #[test]
    fn test_entries_in_range_single() {
        let a = make_analyzer();
        let range = a.entries_in_range(1000, 1000);
        assert_eq!(range.len(), 1);
        assert_eq!(range[0].level, "INFO");
    }
}

fn main() {
    let mut analyzer = LogAnalyzer::new();

    let sample_entries = vec![
        LogEntry::new("INFO",  "api",    "server started on :8080",   1708300000),
        LogEntry::new("DEBUG", "api",    "loading route table",        1708300001),
        LogEntry::new("ERROR", "db",     "connection refused",          1708300002),
        LogEntry::new("WARN",  "worker", "queue depth 750",             1708300003),
        LogEntry::new("ERROR", "db",     "connection refused",          1708300004),
        LogEntry::new("INFO",  "worker", "processing job 9812",         1708300005),
        LogEntry::new("ERROR", "worker", "job 9812 failed: timeout",    1708300006),
        LogEntry::new("INFO",  "api",    "GET /health 200",             1708300007),
    ];

    for entry in sample_entries {
        analyzer.ingest(entry);
    }

    println!("Level summary:");
    for (level, count) in analyzer.level_summary() {
        println!("  {level}: {count}");
    }

    println!("\nTop 2 sources:");
    for (source, count) in analyzer.top_sources(2) {
        println!("  {source}: {count} entries");
    }

    println!("\nError entries: {}", analyzer.errors().len());

    println!("\nMost frequent errors:");
    for (msg, count) in analyzer.most_frequent_errors(3) {
        println!("  ({count}x) {msg}");
    }

    println!("\nSources with errors: {:?}", analyzer.sources_with_errors());
}
