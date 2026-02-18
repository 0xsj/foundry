// Log Analyzer — Reference Solution
//
// Run tests:  rustc --test main.rs && ./main
// Run binary: rustc main.rs && ./main

#![allow(dead_code)]

use std::collections::{BTreeMap, BTreeSet, HashMap};

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
    fn new(level: &'static str, source: &'static str, message: &str, timestamp_ms: u64) -> LogEntry {
        LogEntry {
            level,
            source,
            message: message.to_string(),
            timestamp_ms,
        }
    }
}

impl LogAnalyzer {
    fn new() -> LogAnalyzer {
        LogAnalyzer {
            entries: Vec::new(),
            level_counts: HashMap::new(),
            source_counts: HashMap::new(),
        }
    }

    /// Ingests a log entry.
    ///
    /// Key concept: entry API performs a single lookup. `or_insert(0)` returns
    /// `&mut u32` — we dereference it with `*` to increment the actual integer.
    /// Without `*`, we'd be moving the reference, which isn't valid.
    fn ingest(&mut self, entry: LogEntry) {
        *self.level_counts.entry(entry.level).or_insert(0) += 1;
        *self.source_counts.entry(entry.source).or_insert(0) += 1;
        self.entries.push(entry);
    }

    /// Returns level counts in a BTreeMap (sorted by key).
    ///
    /// Key concept: cloning the HashMap into a BTreeMap. We could also collect
    /// directly, but copying the level_counts map is idiomatic for this case.
    fn level_summary(&self) -> BTreeMap<&'static str, u32> {
        self.level_counts.iter().map(|(&k, &v)| (k, v)).collect()
    }

    /// Returns top n sources by count, ties broken alphabetically.
    ///
    /// Key concept: sort_by with a comparator closure. `.then()` chains a
    /// secondary comparison — only used when the primary comparison is Equal.
    fn top_sources(&self, n: usize) -> Vec<(&'static str, u32)> {
        let mut pairs: Vec<(&'static str, u32)> = self.source_counts
            .iter()
            .map(|(&k, &v)| (k, v))
            .collect();

        // Sort by count descending, then by source name ascending for ties
        pairs.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
        pairs.truncate(n);
        pairs
    }

    /// Returns references to ERROR entries, in insertion order.
    ///
    /// Key concept: `iter()` borrows the Vec. The returned `Vec<&LogEntry>`
    /// holds references into `self.entries`. Lifetime is tied to `&self`.
    fn errors(&self) -> Vec<&LogEntry> {
        self.entries.iter().filter(|e| e.level == "ERROR").collect()
    }

    /// Returns top n (message, count) pairs from error entries.
    ///
    /// Key concept: building a local HashMap from filtered entries.
    /// `entry.message.as_str()` converts &String to &str — the lifetime is
    /// tied to `&self`, so these string slices are valid for the return value.
    fn most_frequent_errors(&self, n: usize) -> Vec<(&str, u32)> {
        let mut error_counts: HashMap<&str, u32> = HashMap::new();
        for entry in self.entries.iter().filter(|e| e.level == "ERROR") {
            *error_counts.entry(entry.message.as_str()).or_insert(0) += 1;
        }

        let mut pairs: Vec<(&str, u32)> = error_counts.into_iter().collect();
        pairs.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
        pairs.truncate(n);
        pairs
    }

    /// Returns sorted list of sources that emitted at least one ERROR.
    ///
    /// Key concept: BTreeSet automatically deduplicates and keeps elements
    /// sorted. Collecting filtered entries into a BTreeSet is the idiomatic
    /// way to get a sorted, unique set of values.
    fn sources_with_errors(&self) -> Vec<&'static str> {
        let unique: BTreeSet<&'static str> = self.entries
            .iter()
            .filter(|e| e.level == "ERROR")
            .map(|e| e.source)
            .collect();
        unique.into_iter().collect()
    }

    /// Returns references to entries within the given timestamp range (inclusive).
    ///
    /// Key concept: filter over a borrowed Vec, collecting &T references.
    /// The closure compares timestamps with both bounds to implement inclusive range.
    fn entries_in_range(&self, start_ms: u64, end_ms: u64) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|e| e.timestamp_ms >= start_ms && e.timestamp_ms <= end_ms)
            .collect()
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

    #[test]
    fn test_new_is_empty() {
        let a = LogAnalyzer::new();
        assert_eq!(a.entries.len(), 0);
        assert!(a.level_counts.is_empty());
        assert!(a.source_counts.is_empty());
    }

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

    #[test]
    fn test_level_summary_sorted() {
        let a = make_analyzer();
        let summary = a.level_summary();
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

    #[test]
    fn test_top_sources_n2() {
        let a = make_analyzer();
        let top = a.top_sources(2);
        assert_eq!(top.len(), 2);
        assert_eq!(top[0], ("api", 4));
        assert_eq!(top[1], ("db", 3));
    }

    #[test]
    fn test_top_sources_all() {
        let a = make_analyzer();
        assert_eq!(a.top_sources(100).len(), 3);
    }

    #[test]
    fn test_top_sources_tie_breaking() {
        let mut a = LogAnalyzer::new();
        a.ingest(LogEntry::new("INFO", "zebra", "msg", 1));
        a.ingest(LogEntry::new("INFO", "alpha", "msg", 2));
        a.ingest(LogEntry::new("INFO", "mango", "msg", 3));
        let top = a.top_sources(3);
        assert_eq!(top[0].0, "alpha");
        assert_eq!(top[1].0, "mango");
        assert_eq!(top[2].0, "zebra");
    }

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
        assert_eq!(errors[0].source, "db");
        assert_eq!(errors[0].timestamp_ms, 1001);
        assert_eq!(errors[1].source, "api");
    }

    #[test]
    fn test_most_frequent_errors_top1() {
        let a = make_analyzer();
        let top = a.most_frequent_errors(1);
        assert_eq!(top.len(), 1);
        assert_eq!(top[0].0, "connection refused");
        assert_eq!(top[0].1, 3);
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

    #[test]
    fn test_sources_with_errors() {
        let a = make_analyzer();
        assert_eq!(a.sources_with_errors(), vec!["api", "db", "worker"]);
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
        assert!(a.entries_in_range(9000, 9999).is_empty());
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
