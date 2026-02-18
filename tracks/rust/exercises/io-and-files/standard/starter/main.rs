// Append-Only Key-Value Store — I/O and Files Exercise (Rust)
//
// Build a log-structured key-value store backed by an append-only log file.
//
// Run tests: rustc --test main.rs && ./main

use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufReader, BufWriter, Cursor, Write};
use std::path::Path;

// ---------- Data types ----------

/// A single entry in the append-only log.
/// value = None represents a deletion (tombstone).
pub struct LogEntry {
    pub key: String,
    pub value: Option<String>,
}

// ---------- Serialization ----------

/// Encodes a LogEntry as a single newline-terminated line.
///
/// Format:
///   SET key value\n     (for inserts)
///   DEL key\n           (for tombstones)
pub fn encode_entry(entry: &LogEntry) -> String {
    todo!()
}

/// Parses a single line back into a LogEntry.
/// Returns None for malformed lines — do not panic.
///
/// Expected inputs:
///   "SET user:1001 alice" → LogEntry { key: "user:1001", value: Some("alice") }
///   "DEL user:1002"       → LogEntry { key: "user:1002", value: None }
///   "garbage"             → None
pub fn decode_entry(line: &str) -> Option<LogEntry> {
    todo!()
}

// ---------- LogStore ----------

/// A write-only handle to an append-only log.
/// Generic over W so it works with BufWriter<File> in production
/// and Cursor<Vec<u8>> in tests.
pub struct LogStore<W: Write> {
    writer: W,
}

impl<W: Write> LogStore<W> {
    /// Create a new LogStore wrapping the given writer.
    pub fn new(writer: W) -> Self {
        todo!()
    }

    /// Append a SET entry (insert/update).
    pub fn set(&mut self, key: &str, value: &str) -> io::Result<()> {
        todo!()
    }

    /// Append a DEL entry (tombstone).
    pub fn delete(&mut self, key: &str) -> io::Result<()> {
        todo!()
    }

    /// Flush the underlying writer.
    /// Must be called before the LogStore is dropped to ensure data is written.
    pub fn flush(&mut self) -> io::Result<()> {
        todo!()
    }
}

// ---------- Reading ----------

/// Scan a log and return the current state as a HashMap.
///
/// Rules:
/// - SET entries insert/overwrite the key
/// - DEL entries remove the key
/// - Later entries always win over earlier entries (append-only = last write wins)
/// - Malformed lines are silently skipped (do not return an error)
pub fn read_log(reader: impl BufRead) -> io::Result<HashMap<String, String>> {
    todo!()
}

// ---------- Compaction ----------

/// Produce a compacted log: read input, write only the current live state.
///
/// The compacted output must be in sorted key order (deterministic).
/// Returns the number of entries written to output.
pub fn compact(input: impl BufRead, output: &mut impl Write) -> io::Result<usize> {
    todo!()
}

// ---------- Production entry point ----------

/// Open (or create) a log file and return a LogStore backed by BufWriter<File>.
///
/// The file is opened in append mode — existing entries are preserved.
pub fn open_log(path: impl AsRef<Path>) -> io::Result<LogStore<BufWriter<File>>> {
    todo!()
}

// ---------- Tests ----------

#[cfg(test)]
mod tests {
    use super::*;

    // --- encode / decode round-trip ---

    #[test]
    fn test_encode_decode_set() {
        let entry = LogEntry {
            key: "user:1001".to_string(),
            value: Some("alice".to_string()),
        };
        let line = encode_entry(&entry);
        assert!(line.ends_with('\n'), "encoded line must end with newline");

        let decoded = decode_entry(line.trim()).unwrap();
        assert_eq!(decoded.key, "user:1001");
        assert_eq!(decoded.value.as_deref(), Some("alice"));
    }

    #[test]
    fn test_encode_decode_del() {
        let entry = LogEntry {
            key: "user:1002".to_string(),
            value: None,
        };
        let line = encode_entry(&entry);
        let decoded = decode_entry(line.trim()).unwrap();
        assert_eq!(decoded.key, "user:1002");
        assert!(decoded.value.is_none());
    }

    #[test]
    fn test_decode_malformed_returns_none() {
        assert!(decode_entry("").is_none());
        assert!(decode_entry("UNKNOWN key value").is_none());
        assert!(decode_entry("SET").is_none());         // missing key
        assert!(decode_entry("SET key").is_none());    // missing value for SET
        assert!(decode_entry("DEL").is_none());        // missing key
    }

    #[test]
    fn test_decode_value_with_spaces() {
        // Values may contain spaces — only split on first two spaces
        let decoded = decode_entry("SET flag:1 feature enabled for users").unwrap();
        assert_eq!(decoded.key, "flag:1");
        assert_eq!(decoded.value.as_deref(), Some("feature enabled for users"));
    }

    // --- LogStore writes ---

    #[test]
    fn test_logstore_set_produces_lines() {
        let mut buf = Cursor::new(Vec::new());
        {
            let mut store = LogStore::new(&mut buf);
            store.set("k1", "v1").unwrap();
            store.set("k2", "v2").unwrap();
            store.flush().unwrap();
        }
        let output = String::from_utf8(buf.into_inner()).unwrap();
        assert!(output.contains("k1"));
        assert!(output.contains("v1"));
        assert!(output.contains("k2"));
        assert!(output.contains("v2"));
    }

    #[test]
    fn test_logstore_delete_produces_del_line() {
        let mut buf = Cursor::new(Vec::new());
        {
            let mut store = LogStore::new(&mut buf);
            store.set("k1", "v1").unwrap();
            store.delete("k1").unwrap();
            store.flush().unwrap();
        }
        let output = String::from_utf8(buf.into_inner()).unwrap();
        assert!(output.contains("DEL"));
        assert!(output.contains("k1"));
    }

    // --- read_log ---

    #[test]
    fn test_read_log_basic_state() {
        let log = b"SET user:1 alice\nSET user:2 bob\nSET user:3 carol\n";
        let reader = BufReader::new(Cursor::new(log));
        let state = read_log(reader).unwrap();

        assert_eq!(state.get("user:1").map(String::as_str), Some("alice"));
        assert_eq!(state.get("user:2").map(String::as_str), Some("bob"));
        assert_eq!(state.get("user:3").map(String::as_str), Some("carol"));
    }

    #[test]
    fn test_read_log_later_write_wins() {
        let log = b"SET flag:dark_mode false\nSET flag:dark_mode true\n";
        let reader = BufReader::new(Cursor::new(log));
        let state = read_log(reader).unwrap();
        assert_eq!(state.get("flag:dark_mode").map(String::as_str), Some("true"));
    }

    #[test]
    fn test_read_log_del_removes_key() {
        let log = b"SET user:1 alice\nDEL user:1\n";
        let reader = BufReader::new(Cursor::new(log));
        let state = read_log(reader).unwrap();
        assert!(!state.contains_key("user:1"));
    }

    #[test]
    fn test_read_log_del_then_set() {
        // Delete followed by re-insert — key should survive
        let log = b"SET user:1 alice\nDEL user:1\nSET user:1 alice-reborn\n";
        let reader = BufReader::new(Cursor::new(log));
        let state = read_log(reader).unwrap();
        assert_eq!(state.get("user:1").map(String::as_str), Some("alice-reborn"));
    }

    #[test]
    fn test_read_log_skips_malformed() {
        let log = b"SET user:1 alice\nBADLINE\nSET user:2 bob\n";
        let reader = BufReader::new(Cursor::new(log));
        let state = read_log(reader).unwrap();
        assert_eq!(state.len(), 2);
    }

    #[test]
    fn test_read_log_empty() {
        let reader = BufReader::new(Cursor::new(b""));
        let state = read_log(reader).unwrap();
        assert!(state.is_empty());
    }

    // --- compact ---

    #[test]
    fn test_compact_deduplicates() {
        let log = b"SET k1 v1\nSET k2 v2\nSET k1 v1-updated\n";
        let reader = BufReader::new(Cursor::new(log));

        let mut output = Cursor::new(Vec::new());
        let count = compact(reader, &mut output).unwrap();

        assert_eq!(count, 2); // k1 and k2 — k1's old entry removed
        let out_str = String::from_utf8(output.into_inner()).unwrap();
        // k1 should appear only once with the updated value
        assert_eq!(out_str.matches("k1").count(), 1);
        assert!(out_str.contains("v1-updated"));
    }

    #[test]
    fn test_compact_removes_deleted_keys() {
        let log = b"SET k1 v1\nSET k2 v2\nDEL k1\n";
        let reader = BufReader::new(Cursor::new(log));

        let mut output = Cursor::new(Vec::new());
        let count = compact(reader, &mut output).unwrap();

        assert_eq!(count, 1); // only k2 survives
        let out_str = String::from_utf8(output.into_inner()).unwrap();
        assert!(!out_str.contains("k1"));
        assert!(out_str.contains("k2"));
    }

    #[test]
    fn test_compact_output_is_sorted() {
        let log = b"SET zebra last\nSET alpha first\nSET middle mid\n";
        let reader = BufReader::new(Cursor::new(log));

        let mut output = Cursor::new(Vec::new());
        compact(reader, &mut output).unwrap();

        let out_str = String::from_utf8(output.into_inner()).unwrap();
        let lines: Vec<&str> = out_str.lines().collect();
        assert_eq!(lines.len(), 3);
        // Sorted: alpha, middle, zebra
        assert!(lines[0].contains("alpha"));
        assert!(lines[1].contains("middle"));
        assert!(lines[2].contains("zebra"));
    }

    // --- round-trip integration ---

    #[test]
    fn test_write_then_read_roundtrip() {
        let mut buf = Cursor::new(Vec::new());

        // Write phase
        {
            let mut store = LogStore::new(&mut buf);
            store.set("config:timeout", "30s").unwrap();
            store.set("config:retries", "3").unwrap();
            store.set("config:timeout", "60s").unwrap(); // override
            store.delete("config:retries").unwrap();      // remove
            store.set("config:debug", "false").unwrap();
            store.flush().unwrap();
        }

        // Read phase — rewind cursor before reading
        buf.set_position(0);
        let state = read_log(BufReader::new(buf)).unwrap();

        assert_eq!(state.get("config:timeout").map(String::as_str), Some("60s"));
        assert!(!state.contains_key("config:retries")); // deleted
        assert_eq!(state.get("config:debug").map(String::as_str), Some("false"));
        assert_eq!(state.len(), 2);
    }
}

fn main() {
    println!("Append-Only Key-Value Store");
    println!("Run: rustc --test main.rs && ./main");
}
