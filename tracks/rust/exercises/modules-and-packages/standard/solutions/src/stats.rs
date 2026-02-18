// url-shortener/src/stats.rs
//
// The stats module tracks operation counts for analytics.
//
// Key decision: the counter fields are private. External code calls
// `record_shorten()` etc. instead of directly incrementing `shortened_count`.
// This means:
//   1. The format of `report()` can change without updating callers
//   2. Stats validation logic (e.g., "assert count never goes negative")
//      can be added here without any API changes
//   3. The field names/types are free to change

/// Tracks operation counts for CLI analytics.
pub struct Stats {
    shortened_count: u64,
    resolved_count: u64,
    removed_count: u64,
}

impl Stats {
    /// Create a new Stats tracker with all counters at zero.
    pub fn new() -> Stats {
        Stats {
            shortened_count: 0,
            resolved_count: 0,
            removed_count: 0,
        }
    }

    /// Record a successful shorten operation.
    pub fn record_shorten(&mut self) {
        self.shortened_count += 1;
    }

    /// Record a successful resolve operation.
    pub fn record_resolve(&mut self) {
        self.resolved_count += 1;
    }

    /// Record a successful remove operation.
    pub fn record_remove(&mut self) {
        self.removed_count += 1;
    }

    /// Returns a formatted summary of all operation counts.
    pub fn report(&self) -> String {
        format!(
            "Shortened: {}  Resolved: {}  Removed: {}",
            self.shortened_count, self.resolved_count, self.removed_count
        )
    }
}

impl Default for Stats {
    fn default() -> Stats {
        Stats::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_report() {
        let stats = Stats::new();
        assert_eq!(stats.report(), "Shortened: 0  Resolved: 0  Removed: 0");
    }

    #[test]
    fn test_record_shorten() {
        let mut stats = Stats::new();
        stats.record_shorten();
        stats.record_shorten();
        assert_eq!(stats.report(), "Shortened: 2  Resolved: 0  Removed: 0");
    }

    #[test]
    fn test_record_mixed() {
        let mut stats = Stats::new();
        stats.record_shorten();
        stats.record_shorten();
        stats.record_resolve();
        stats.record_remove();

        assert_eq!(stats.report(), "Shortened: 2  Resolved: 1  Removed: 1");
    }
}
